# benchmark notes

this is the long version of the benchmark. the short version, and how to
build and run everything, is in the [readme](../README.org).

## what is in the repo

| path | language | role |
|------|----------|------|
| `src/main.rs` | rust | interactive tool (`minifb` + `image` crate) |
| `src/bin/circular-bench.rs` | rust | head-less benchmark binary (shared codec via ffi) |
| `c/main.c` | c | interactive tool (SDL2) and head-less mode |
| `c/circular-bench-c` | c | head-less-only binary (no SDL2, for fair startup) |
| `nim/circular_img.nim` | nim | head-less benchmark binary (shared codec via ffi) |
| `common/` | c | the shared codec linked by all three |
| `bench.py` | python | runs and ranks the implementations |

## how the benchmark stays fair

- every port links the same `common/libcodec.a` (stb_image + stb_image_write),
  so decoding the jpeg and encoding the png are identical bytes of work. the
  saved pngs are byte-identical across all three languages.
- each language's head-less binary is timed: `circular-bench` (rust),
  `circular-bench-c` (c, built without SDL2 so startup matches the others) and
  `nim/circular-img`.
- `bench.py` does 2 untimed warm-up runs per binary, then the timed runs, and
  ranks by median.
- wall-clock time wraps the whole process, so process startup counts too.

why the warm-up matters: the first launch of a freshly signed binary on macos
can stall for hundreds of milliseconds while the os verifies its signature (we
saw a single 622 ms outlier). linking a gui library into a "head-less" binary
also adds a few ms of dyld work, which is why the c benchmark target leaves
SDL2 out.

## raw numbers

machine: apple silicon (arm64), macos. input: `wallp-1.jpg` (1280x720),
`cx=640 cy=280 radius=220`.

### end to end (open + crop + save), 3 timed runs

| rank | language | median | best | runs (ms) |
|------|----------|--------|------|-----------|
| 1 | c | 31.35 ms | 31.30 ms | 31.3 / 31.6 / 31.4 |
| 2 | rust | 31.64 ms | 31.52 ms | 31.8 / 31.6 / 31.5 |
| 3 | nim | 32.10 ms | 31.99 ms | 32.0 / 32.3 / 32.1 |

phase medians: decode about 5.42 ms for all three (same codec), save about
25 ms for all three (same codec). the whole end-to-end spread is under 0.8 ms,
about 2.4 percent.

### crop loop amplified (`CIRCULAR_REPEAT=200`, 5 timed runs)

| rank | language | total median | crop median | per crop |
|------|----------|--------------|-------------|----------|
| 1 | c | 147.9 ms | 117.0 ms | 0.585 ms |
| 2 | rust | 176.4 ms | 145.2 ms | 0.726 ms |
| 3 | nim | 183.6 ms | 152.3 ms | 0.761 ms |

## what it means

### end to end is a codec benchmark

with the same codec, decode and save are identical and dominate: about 30 ms
of the 31 ms total. so all three languages land within about 2.4 percent of
each other. if you instead compare "rust's `image` crate vs c's stb" the codec
choice swamps the language by roughly 3x. that is why this repo pins one codec.

### the crop loop is the language part

the per-pixel crop loop is the only real language work. over 200 iterations c
is about 20 percent faster than safe rust and about 30 percent faster than nim.
all three compile through llvm/clang, so the difference is runtime and
bounds-check overhead, not code generation.

### rust's gap is bounds checking

swapping the four hot slice writes to `get_unchecked` (a local experiment, not
committed - the repo keeps the safe version) dropped rust's crop from 0.726 to
about 0.628 ms per call, within roughly 7 percent of c. nim pays a similar but
smaller penalty.

## advice

- for this workload the language barely matters, the codecs do. choose based on
  the codec ecosystem and developer fit, not on these numbers.
- if the crop ever became hot (huge images, video, batch jobs), c has a small
  real edge, and safe rust is close enough that unchecked or iterator-based
  indexing erases most of the gap.
- nim is last on the isolated loop but only by about 30 percent, which is a
  fine trade for its ergonomics.
