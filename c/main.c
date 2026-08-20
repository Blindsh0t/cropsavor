// C port of the circular-img tool.
//
// Same behaviour as the Rust version:
//   * opens a JPEG/PNG, lets you drag out a circle, saves an anti-aliased
//     circular crop centred on a padded transparent square canvas.
//   * with <input> <cx> <cy> <radius> it runs head-less for benchmarking.
//
// Image loading/encoding uses the shared codec in ../common so the benchmark
// compares languages rather than image libraries. The window and mouse
// handling use SDL2 (mirroring minifb in the Rust version).

#include "../common/codec.h"

#ifndef CIMG_HEADLESS_ONLY
#include <SDL2/SDL.h>
#endif

#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

// ---------------------------------------------------------------------------
// Save path
// ---------------------------------------------------------------------------

// Build "<stem>_circular.png" next to the input file.
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
// Interactive preview
// ---------------------------------------------------------------------------
#ifndef CIMG_HEADLESS_ONLY

// Composite the RGBA image over a checkerboard into 0x00RRGGBB values.
static uint32_t *build_display_buffer(const CImage *image) {
    size_t count = (size_t)image->width * image->height;
    uint32_t *buffer = malloc(count * sizeof(uint32_t));

    for (int y = 0; y < image->height; y++) {
        for (int x = 0; x < image->width; x++) {
            const unsigned char *pixel =
                &image->rgba[((size_t)y * image->width + x) * 4];

            uint32_t checkerboard = (((x / 16) + (y / 16)) % 2 == 0) ? 220u : 170u;

            uint32_t alpha = pixel[3];

            uint32_t r = ((uint32_t)pixel[0] * alpha + checkerboard * (255 - alpha)) / 255;
            uint32_t g = ((uint32_t)pixel[1] * alpha + checkerboard * (255 - alpha)) / 255;
            uint32_t b = ((uint32_t)pixel[2] * alpha + checkerboard * (255 - alpha)) / 255;

            buffer[(size_t)y * image->width + x] = (r << 16) | (g << 8) | b;
        }
    }

    return buffer;
}

static void draw_pixel(uint32_t *buffer, int width, int height, int x, int y,
                       uint32_t color) {
    if (x >= 0 && x < width && y >= 0 && y < height) {
        buffer[(size_t)y * width + x] = color;
    }
}

static void draw_circle_preview(uint32_t *buffer, int width, int height,
                                float cx, float cy, float radius) {
    int center_x = (int)lroundf(cx);
    int center_y = (int)lroundf(cy);

    // Draw the circle outline.
    const int steps = 720;

    for (int i = 0; i < steps; i++) {
        float angle = (float)i / (float)steps * (float)(M_PI * 2.0);

        int x = (int)lroundf(cx + radius * cosf(angle));
        int y = (int)lroundf(cy + radius * sinf(angle));

        draw_pixel(buffer, width, height, x, y, 0x00FF3300);
    }

    // Draw a line from the center to the mouse cursor.
    int end_x = (int)lroundf(cx + radius);

    int line_length = abs(end_x - center_x);
    if (line_length < 1) line_length = 1;

    for (int i = 0; i <= line_length; i++) {
        int x = (end_x >= center_x) ? center_x + i : center_x - i;

        draw_pixel(buffer, width, height, x, center_y, 0x00FFCC00);
    }

    // Draw a crosshair at the fixed center.
    for (int offset = -8; offset <= 8; offset++) {
        draw_pixel(buffer, width, height, center_x + offset, center_y, 0x0000FF00);
        draw_pixel(buffer, width, height, center_x, center_y + offset, 0x0000FF00);
    }

    // Draw a small marker at the radius endpoint.
    draw_pixel(buffer, width, height, end_x, center_y, 0x00FFFFFF);
}

#endif // CIMG_HEADLESS_ONLY

// ---------------------------------------------------------------------------
// Modes
// ---------------------------------------------------------------------------

static int env_int(const char *name, int fallback) {
    const char *value = getenv(name);
    if (!value || !*value) return fallback;

    int parsed = atoi(value);
    return parsed > 0 ? parsed : fallback;
}

static int run_headless(const char *input_path, float cx, float cy,
                        float radius) {
    int timing = getenv("CIRCULAR_TIMING") != NULL;
    int repeat = env_int("CIRCULAR_REPEAT", 1);

    struct timespec t0, t1, t2, t3;
    clock_gettime(CLOCK_MONOTONIC, &t0);
    CImage image = cimg_load(input_path);
    clock_gettime(CLOCK_MONOTONIC, &t1);

    if (!image.rgba) {
        fprintf(stderr, "Failed to load image: %s\n", input_path);
        return 1;
    }

    CImage cropped = {0, 0, NULL};

    for (int i = 0; i < repeat; i++) {
        cimg_free(&cropped);
        cropped = crop_to_circle(&image, cx, cy, radius);
    }
    clock_gettime(CLOCK_MONOTONIC, &t2);

    char path[4096];
    output_path(input_path, path, sizeof(path));
    cimg_save_png(path, &cropped);
    clock_gettime(CLOCK_MONOTONIC, &t3);

    if (timing) {
        double ms = 1e3;
        double d0 = (t1.tv_sec - t0.tv_sec) + (t1.tv_nsec - t0.tv_nsec) / 1e9;
        double d1 = (t2.tv_sec - t1.tv_sec) + (t2.tv_nsec - t1.tv_nsec) / 1e9;
        double d2 = (t3.tv_sec - t2.tv_sec) + (t3.tv_nsec - t2.tv_nsec) / 1e9;
        fprintf(stderr,
                "[c] decode %.2f ms | crop %.2f ms (%.4f ms x%d) | save %.2f ms | total %.2f ms\n",
                d0 * ms, d1 * ms, d1 * ms / repeat, repeat, d2 * ms,
                (d0 + d1 + d2) * ms);
    }

    printf("Saved circular image to:\n%s\n", path);

    cimg_free(&cropped);
    cimg_free(&image);

    return 0;
}

#ifndef CIMG_HEADLESS_ONLY
static int run_interactive(const char *input_path) {
    CImage image = cimg_load(input_path);

    if (!image.rgba) {
        fprintf(stderr, "Failed to load image: %s\n", input_path);
        return 1;
    }

    if (SDL_Init(SDL_INIT_VIDEO) != 0) {
        fprintf(stderr, "SDL_Init failed: %s\n", SDL_GetError());
        return 1;
    }

    SDL_Window *window = SDL_CreateWindow("Circular Crop - click and drag, Esc to exit",
                                          SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
                                          image.width, image.height, 0);
    if (!window) {
        fprintf(stderr, "SDL_CreateWindow failed: %s\n", SDL_GetError());
        return 1;
    }

    SDL_Renderer *renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);
    SDL_Texture *texture =
        SDL_CreateTexture(renderer, SDL_PIXELFORMAT_ARGB8888,
                          SDL_TEXTUREACCESS_STREAMING, image.width, image.height);

    uint32_t *original_buffer = build_display_buffer(&image);
    uint32_t *display_buffer = malloc((size_t)image.width * image.height * sizeof(uint32_t));
    memcpy(display_buffer, original_buffer, (size_t)image.width * image.height * sizeof(uint32_t));

    int has_center = 0;
    float center_x = 0.0f, center_y = 0.0f;
    float radius = 0.0f;
    int dragging = 0;
    int mouse_was_down = 0;
    int quit = 0;

    while (!quit) {
        SDL_Event event;
        while (SDL_PollEvent(&event)) {
            if (event.type == SDL_QUIT) {
                quit = 1;
            }
            if (event.type == SDL_KEYDOWN && event.key.keysym.sym == SDLK_ESCAPE) {
                quit = 1;
            }
        }

        int mouse_x = 0, mouse_y = 0;
        int mouse_down = SDL_GetMouseState(&mouse_x, &mouse_y) & SDL_BUTTON(SDL_BUTTON_LEFT);

        // Mouse button was just pressed.
        if (mouse_down && !mouse_was_down) {
            center_x = (float)mouse_x;
            center_y = (float)mouse_y;
            has_center = 1;
            radius = 0.0f;
            dragging = 1;
        }

        // Mouse is being dragged.
        if (dragging && mouse_down && has_center) {
            float dx = (float)mouse_x - center_x;
            float dy = (float)mouse_y - center_y;

            radius = sqrtf(dx * dx + dy * dy);

            memcpy(display_buffer, original_buffer,
                   (size_t)image.width * image.height * sizeof(uint32_t));

            draw_circle_preview(display_buffer, image.width, image.height,
                                center_x, center_y, radius);
        }

        // Mouse button was just released.
        if (!mouse_down && mouse_was_down && dragging) {
            if (has_center && radius > 1.0f) {
                CImage cropped = crop_to_circle(&image, center_x, center_y, radius);

                char path[4096];
                output_path(input_path, path, sizeof(path));
                cimg_save_png(path, &cropped);

                printf("Saved circular image to:\n%s\n", path);

                cimg_free(&cropped);
            }

            dragging = 0;
        }

        mouse_was_down = mouse_down;

        SDL_UpdateTexture(texture, NULL, display_buffer, image.width * 4);
        SDL_RenderClear(renderer);
        SDL_RenderCopy(renderer, texture, NULL, NULL);
        SDL_RenderPresent(renderer);
    }

    free(display_buffer);
    free(original_buffer);
    SDL_DestroyTexture(texture);
    SDL_DestroyRenderer(renderer);
    SDL_DestroyWindow(window);
    SDL_Quit();
    cimg_free(&image);

    return 0;
}
#endif // CIMG_HEADLESS_ONLY

int main(int argc, char **argv) {
    const char *input_path = "/Users/han/Desktop/wallp-1.jpg";

    if (argc >= 2) {
        input_path = argv[1];
    }

    // Head-less mode used for benchmarking: input cx cy radius
    if (argc >= 5) {
        float cx = strtof(argv[2], NULL);
        float cy = strtof(argv[3], NULL);
        float radius = strtof(argv[4], NULL);

        return run_headless(input_path, cx, cy, radius);
    }

#ifndef CIMG_HEADLESS_ONLY
    return run_interactive(input_path);
#else
    fprintf(stderr, "Usage: %s <input> <cx> <cy> <radius>\n", argv[0]);
    return 2;
#endif
}
