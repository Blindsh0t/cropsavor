# Nim port of the circular-img head-less pipeline.
#
# Links the shared C codec in ../common so decode/encode are identical to the
# C and Rust ports; only the language-specific crop loop differs.
#
#   circular-img <input> <cx> <cy> <radius>
#
# Env:
#   CIRCULAR_TIMING=1   print decode / crop / save breakdown
#   CIRCULAR_REPEAT=N   run the crop N times (default 1)

import std/[math, monotimes, os, strutils, times]

type
  CImage {.importc: "CImage", header: "codec.h", bycopy.} = object
    width: cint
    height: cint
    rgba: ptr uint8

proc cimg_load(path: cstring): CImage {.importc, header: "codec.h".}
proc cimg_save_png(path: cstring, image: ptr CImage): cint {.importc, header: "codec.h".}
proc cimg_free(image: ptr CImage) {.importc, header: "codec.h".}

proc outputPath(inputPath: string): string =
  let directory = parentDir(inputPath)
  let name = splitFile(inputPath).name
  if directory.len == 0:
    name & "_circular.png"
  else:
    directory / (name & "_circular.png")

proc ms(start, finish: MonoTime): float =
  float((finish - start).inNanoseconds) / 1e6

# Cut a circle out of the source with an anti-aliased edge.
proc cropToCircle(source: CImage; cx, cy, radius: float32): tuple[pixels: ptr uint8, size: int] =
  let size = int(source.width)
  let buffer = cast[ptr UncheckedArray[uint8]](alloc0(size * size * 4))
  let src = cast[ptr UncheckedArray[uint8]](source.rgba)

  for y in 0 ..< size:
    for x in 0 ..< size:
      let dx = float32(x) + 0.5'f32 - cx
      let dy = float32(y) + 0.5'f32 - cy

      let distance = sqrt(dx * dx + dy * dy)

      var coverage = radius - distance + 0.5'f32
      if coverage < 0.0'f32: coverage = 0.0'f32
      if coverage > 1.0'f32: coverage = 1.0'f32

      if coverage <= 0.0'f32:
        continue

      let offset = (y * size + x) * 4
      buffer[offset] = src[offset]
      buffer[offset + 1] = src[offset + 1]
      buffer[offset + 2] = src[offset + 2]
      buffer[offset + 3] = uint8(round(float32(src[offset + 3]) * coverage))

  result = (cast[ptr uint8](buffer), size)

proc envInt(name: string, fallback: int): int =
  if existsEnv(name):
    let parsed = parseInt(getEnv(name))
    if parsed > 0: parsed else: fallback
  else:
    fallback

proc main() =
  var inputPath = "/Users/han/Desktop/wallp-1.jpg"
  var cx = 640.0'f32
  var cy = 280.0'f32
  var radius = 220.0'f32

  let args = commandLineParams()
  if args.len >= 1: inputPath = args[0]
  if args.len >= 4:
    cx = parseFloat(args[1]).float32
    cy = parseFloat(args[2]).float32
    radius = parseFloat(args[3]).float32

  let timing = existsEnv("CIRCULAR_TIMING")
  let repeat = envInt("CIRCULAR_REPEAT", 1)

  let t0 = getMonoTime()
  let image = cimg_load(inputPath.cstring)
  let t1 = getMonoTime()

  if image.rgba == nil:
    stderr.writeLine("Failed to load image: " & inputPath)
    quit(1)

  var pixels: ptr uint8 = nil
  var size = 0

  for _ in 0 ..< repeat:
    if pixels != nil:
      dealloc(pixels)
    let cropped = cropToCircle(image, cx, cy, radius)
    pixels = cropped.pixels
    size = cropped.size

  let t2 = getMonoTime()

  let savePath = outputPath(inputPath)
  var outImage = CImage(width: cint(size), height: cint(size), rgba: pixels)
  discard cimg_save_png(savePath.cstring, addr outImage)

  let t3 = getMonoTime()

  if timing:
    let decode = ms(t0, t1)
    let crop = ms(t1, t2)
    let save = ms(t2, t3)
    stderr.writeLine("[nim] decode " & formatFloat(decode, ffDecimal, 2) &
      " ms | crop " & formatFloat(crop, ffDecimal, 2) & " ms (" &
      formatFloat(crop / float(repeat), ffDecimal, 4) & " ms x" & $repeat &
      ") | save " & formatFloat(save, ffDecimal, 2) & " ms | total " &
      formatFloat(decode + crop + save, ffDecimal, 2) & " ms")

  echo "Saved circular image to:"
  echo savePath

  if pixels != nil:
    dealloc(pixels)
  var mutableImage = image
  cimg_free(addr mutableImage)

main()
