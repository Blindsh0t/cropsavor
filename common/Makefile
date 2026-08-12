# Build the shared codec object consumed by all three language ports.
#
#   make        -> libcodec.a
#   make clean

CC     ?= cc
CFLAGS ?= -O2 -std=c11 -Wall -Wextra

LIB := libcodec.a
OBJ := codec.o

.PHONY: all clean

all: $(LIB)

$(LIB): $(OBJ)
	ar rcs $@ $^

$(OBJ): codec.c codec.h vendor/stb_image.h vendor/stb_image_write.h
	$(CC) $(CFLAGS) -c -o $@ codec.c

clean:
	rm -f $(OBJ) $(LIB)
