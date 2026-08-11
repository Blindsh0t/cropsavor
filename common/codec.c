// Shared codec implementation (stb_image + stb_image_write).
//
// Every language port links this same object, so decode/encode cost is
// identical and the benchmark measures only language-specific work.

#include "codec.h"

#define STB_IMAGE_IMPLEMENTATION
#include "vendor/stb_image.h"

#define STB_IMAGE_WRITE_IMPLEMENTATION
#include "vendor/stb_image_write.h"

#include <stddef.h>

CImage cimg_load(const char *path) {
    CImage image = {0, 0, NULL};
    int channels = 0;

    image.rgba = stbi_load(path, &image.width, &image.height, &channels, 4);

    return image;
}

int cimg_save_png(const char *path, const CImage *image) {
    return stbi_write_png(path, image->width, image->height, 4, image->rgba,
                          image->width * 4);
}

void cimg_free(CImage *image) {
    if (image->rgba) {
        stbi_image_free(image->rgba);
    }
    image->rgba = NULL;
    image->width = 0;
    image->height = 0;
}
