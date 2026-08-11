#ifndef CIMG_CODEC_H
#define CIMG_CODEC_H

// Shared image codec used by the C, Rust and Nim implementations so that a
// benchmark compares the languages, not their image libraries.

typedef struct {
    int width;
    int height;
    unsigned char *rgba; // width * height * 4 bytes, RGBA order
} CImage;

// Decode `path` (JPEG/PNG/...) into RGBA. On failure `rgba` is NULL and
// width/height are 0.
CImage cimg_load(const char *path);

// Encode `image` as a PNG. Returns non-zero on success.
int cimg_save_png(const char *path, const CImage *image);

// Free the pixel buffer and reset the struct.
void cimg_free(CImage *image);

#endif // CIMG_CODEC_H
