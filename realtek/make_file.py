#!/usr/bin/env python3
from struct import unpack, pack, iter_unpack
from sys import argv, stdout
header = bytearray(open(argv[1], 'rb').read()[:0x20])
body = bytearray(open(argv[2], 'rb').read())
body += bytes([0] * (len(body) % 2))
header[0xc:0xe] = pack('<H', len(body) + 2)

m = 0
for (short,) in iter_unpack('<H', body):
    m ^= short

appendix = pack('<H', m)

result = header + body + appendix
stdout.buffer.write(result)
