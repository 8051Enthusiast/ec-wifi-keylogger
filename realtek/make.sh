#!/bin/sh
set -eu
asem main.a51
hexbin main.hex
./make_file.py rtl8821aefw_29.bin main.bin > out
