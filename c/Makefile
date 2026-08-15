# Build the C port of circular-img.
#
#   make
#   make clean
#
# SDL2 is used only for the interactive window. If pkg-config's SDL2 does not
# match the host architecture, setup_sdl2.sh assembles a native dylib under
# ./build (see that script for details).

CC      ?= cc
CFLAGS  ?= -O2 -std=c11 -Wall -Wextra
ARCH    := $(shell uname -m)

BIN     := circular-img-c
SRC     := main.c
DEP     := ../common/codec.h ../common/libcodec.a

PKG_SDL_CFLAGS := $(shell pkg-config --cflags sdl2 2>/dev/null)
PKG_SDL_LIBS   := $(shell pkg-config --libs sdl2 2>/dev/null)
PKG_SDL_LIBDIR := $(shell pkg-config --variable=libdir sdl2 2>/dev/null)

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

.PHONY: all clean

all: $(BIN)

$(BIN): $(SRC) $(DEP) $(SDL_EXTRA)
	$(CC) $(CFLAGS) $(SDL_CFLAGS) -o $@ $(SRC) $(LDLIBS) $(SDL_RPATH)
	codesign --force --sign - $@ 2>/dev/null || true

../common/libcodec.a:
	$(MAKE) -C ../common

build/libSDL2.dylib:
	./setup_sdl2.sh

clean:
	rm -f $(BIN)
	rm -rf build
