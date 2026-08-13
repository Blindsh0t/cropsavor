#include "../common/codec.h"

#include <stdio.h>

int main(int argc, char **argv) {
    const char *input_path = "/Users/han/Desktop/wallp-1.jpg";

    if (argc >= 2) {
        input_path = argv[1];
    }

    CImage image = cimg_load(input_path);

    if (!image.rgba) {
        fprintf(stderr, "Failed to load image: %s\n", input_path);
        return 1;
    }

    printf("loaded %dx%d\n", image.width, image.height);

    cimg_free(&image);

    return 0;
}
