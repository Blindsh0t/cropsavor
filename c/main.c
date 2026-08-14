#include "../common/codec.h"

#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// ---------------------------------------------------------------------------
// Save path
// ---------------------------------------------------------------------------

static void output_path(const char *input_path, char *out, size_t out_size) {
    const char *slash = strrchr(input_path, '/');
    const char *basename = slash ? slash + 1 : input_path;

    const char *dot = strrchr(basename, '.');
    size_t stem_len = dot ? (size_t)(dot - basename) : strlen(basename);

    size_t dir_len = slash ? (size_t)(slash - input_path) + 1 : 0;

    snprintf(out, out_size, "%.*s%.*s_circular.png", (int)dir_len, input_path,
             (int)stem_len, basename);
}

// ---------------------------------------------------------------------------
// Circular crop
// ---------------------------------------------------------------------------

// Cut a circular region out of `source`, leaving everything outside the
// circle transparent.
static CImage crop_to_circle(const CImage *source, float cx, float cy,
                             float radius) {
    CImage output = {source->width, source->height, NULL};
    output.rgba = malloc((size_t)output.width * output.height * 4);
    memcpy(output.rgba, source->rgba, (size_t)output.width * output.height * 4);

    float radius_squared = radius * radius;

    for (int y = 0; y < output.height; y++) {
        for (int x = 0; x < output.width; x++) {
            float dx = (float)x - cx;
            float dy = (float)y - cy;

            if (dx * dx + dy * dy > radius_squared) {
                output.rgba[((size_t)y * output.width + x) * 4 + 3] = 0;
            }
        }
    }

    return output;
}

// ---------------------------------------------------------------------------
// Modes
// ---------------------------------------------------------------------------

static int run_headless(const char *input_path, float cx, float cy,
                        float radius) {
    CImage image = cimg_load(input_path);

    if (!image.rgba) {
        fprintf(stderr, "Failed to load image: %s\n", input_path);
        return 1;
    }

    CImage cropped = crop_to_circle(&image, cx, cy, radius);

    char path[4096];
    output_path(input_path, path, sizeof(path));
    cimg_save_png(path, &cropped);

    printf("Saved circular image to:\n%s\n", path);

    cimg_free(&cropped);
    cimg_free(&image);

    return 0;
}

int main(int argc, char **argv) {
    const char *input_path = "/Users/han/Desktop/wallp-1.jpg";

    if (argc >= 2) {
        input_path = argv[1];
    }

    // Head-less mode: input cx cy radius
    if (argc >= 5) {
        float cx = strtof(argv[2], NULL);
        float cy = strtof(argv[3], NULL);
        float radius = strtof(argv[4], NULL);

        return run_headless(input_path, cx, cy, radius);
    }

    fprintf(stderr, "Usage: %s <input> <cx> <cy> <radius>\n", argv[0]);
    return 2;
}
