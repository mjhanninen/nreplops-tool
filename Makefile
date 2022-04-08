.PHONY: all

all: man/nr.1

man/nr.1: man/nr.1.md
	pandoc -f markdown -t man -o $@ $<
