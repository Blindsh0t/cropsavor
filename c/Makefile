# Build the C port of circular-img.
#
#   make
#   make clean

CC      ?= cc
CFLAGS  ?= -O2 -std=c11 -Wall -Wextra

BIN     := circular-img-c
SRC     := main.c
DEP     := ../common/codec.h ../common/libcodec.a

LDLIBS  := ../common/libcodec.a -lm

.PHONY: all clean

all: $(BIN)

$(BIN): $(SRC) $(DEP)
	$(CC) $(CFLAGS) -o $@ $(SRC) $(LDLIBS)

../common/libcodec.a:
	$(MAKE) -C ../common

clean:
	rm -f $(BIN)
