EXE = Burgentruck
LXE = burgentruck
VERSION = ???
TMPDIR := $(CURDIR)/tmp

ifeq ($(OS),Windows_NT)
    INF := win
    EXT := .exe
    RMFILE := del
    RMDIR := rmdir /s /q
    MKDIR := mkdir
    NAME := $(EXE).exe
else
    INF := linux
    EXT :=
    RMFILE := rm -f
    RMDIR := rm -rf
    MKDIR := mkdir -p
    NAME := $(EXE)
endif

openbench:
	cargo rustc -r -- -C target-cpu=native --emit link=$(NAME)

tmp-dir:
	$(MKDIR) $(TMPDIR)

bench:
	cargo rustc --release -- -C target-cpu=native --emit link=$(NAME)
	target/release/$(NAME) bench