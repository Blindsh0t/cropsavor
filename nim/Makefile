# Build the Nim port.
#
#   make        -> ./circular-img
#   make clean
#
# Links the shared codec in ../common/libcodec.a so decode/encode match the
# C and Rust ports exactly.

NIM     ?= nim
NIMFLAGS ?= -d:release --opt:speed --passC:-I../common --passL:../common/libcodec.a

BIN := circular-img

.PHONY: all clean

all: $(BIN)

$(BIN): circular_img.nim ../common/libcodec.a ../common/codec.h
	$(NIM) c $(NIMFLAGS) -o:$(BIN) circular_img.nim

../common/libcodec.a:
	$(MAKE) -C ../common

clean:
	rm -f $(BIN)
	rm -rf nimcache
