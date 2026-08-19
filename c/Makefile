# Build the C port of circular-img.
#
#   make          -> optimized ./circular-img-c
#   make debug    -> -O0 -g build
#   make clean
#
# SDL2 is used only for the interactive window. If pkg-config's SDL2 does not
# match the host architecture, setup_sdl2.sh assembles a native dylib under
# ./build (see that script for details).

CC      ?= cc
CFLAGS  ?= -O2 -std=c11 -Wall -Wextra
ARCH    := $(shell uname -m)

BIN     := circular-img-c
BENCH   := circular-bench-c
SRC     := main.c
DEP     := ../common/codec.h ../common/libcodec.a

PKG_SDL_CFLAGS := $(shell pkg-config --cflags sdl2 2>/dev/null)
PKG_SDL_LIBS   := $(shell pkg-config --libs sdl2 2>/dev/null)
PKG_SDL_LIBDIR := $(shell pkg-config --variable=libdir sdl2 2>/dev/null)

# Only trust pkg-config when its dylib matches the host architecture (or we
# are not on macOS, where this is not a concern).
ifeq ($(shell uname -s),Darwin)
PKG_SDL_OK := $(shell test -n "$(PKG_SDL_LIBS)" && file "$(PKG_SDL_LIBDIR)"/libSDL2*.dylib 2>/dev/null | grep -q "$(ARCH)" && echo yes)
else
PKG_SDL_OK := $(shell test -n "$(PKG_SDL_LIBS)" && echo yes)
endif

ifeq ($(PKG_SDL_OK),yes)
SDL_CFLAGS := $(PKG_SDL_CFLAGS)
SDL_LIBS   := $(PKG_SDL_LIBS)
SDL_RPATH  :=
SDL_EXTRA  :=
else
SDL_CFLAGS := -I/usr/local/include/SDL2 -I/opt/homebrew/include/SDL2
SDL_LIBS   := -Lbuild -lSDL2
SDL_RPATH  := -Wl,-rpath,@executable_path/build
SDL_EXTRA  := build/libSDL2.dylib
endif

LDLIBS  := $(SDL_LIBS) ../common/libcodec.a -lm

.PHONY: all debug clean

all: $(BIN) $(BENCH)

$(BIN): $(SRC) $(DEP) $(SDL_EXTRA)
	$(CC) $(CFLAGS) $(SDL_CFLAGS) -o $@ $(SRC) $(LDLIBS) $(SDL_RPATH)
	codesign --force --sign - $@ 2>/dev/null || true

# Head-less-only binary: no SDL2, so benchmark startup matches the Rust/Nim
# benchmark binaries.
$(BENCH): $(SRC) $(DEP)
	$(CC) $(CFLAGS) -DCIMG_HEADLESS_ONLY -o $@ $(SRC) ../common/libcodec.a -lm
	codesign --force --sign - $@ 2>/dev/null || true

../common/libcodec.a:
	$(MAKE) -C ../common

build/libSDL2.dylib:
	./setup_sdl2.sh

debug: CFLAGS := -O0 -g -std=c11 -Wall -Wextra
debug: $(BIN)

clean:
	rm -f $(BIN) $(BENCH)
	rm -rf build
