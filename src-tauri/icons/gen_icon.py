import struct
import zlib
import os

dir_path = os.path.dirname(os.path.abspath(__file__))

# --- Generate 16x16 icon.ico ---
w, h = 16, 16
info = struct.pack('<IiiHHIIiiII', 40, w, h * 2, 1, 32, 0, 0, 0, 0, 0, 0)
px = b'\xf6\x82\x3b\xff' * w * h
mask = b'\x00' * h * 4
img = info + px + mask
hdr = struct.pack('<HHH', 0, 1, 1)
d = struct.pack('<BBBBHHII', w, h, 0, 0, 1, 32, len(img), 22)
ico_path = os.path.join(dir_path, 'icon.ico')
with open(ico_path, 'wb') as f:
    f.write(hdr + d + img)
print(f'Created {ico_path}')

# --- Generate 32x32 icon.png ---
w2, h2 = 32, 32
raw = b''
for _ in range(h2):
    raw += b'\x00' + b'\x3b\x82\xf6\xff' * w2

ihdr_data = struct.pack('>IIBBBBB', w2, h2, 8, 6, 0, 0, 0)

def png_chunk(chunk_type, data):
    crc = zlib.crc32(chunk_type + data) & 0xffffffff
    return struct.pack('>I', len(data)) + chunk_type + data + struct.pack('>I', crc)

png_path = os.path.join(dir_path, 'icon.png')
with open(png_path, 'wb') as f:
    f.write(b'\x89PNG\r\n\x1a\n')
    f.write(png_chunk(b'IHDR', ihdr_data))
    f.write(png_chunk(b'IDAT', zlib.compress(raw)))
    f.write(png_chunk(b'IEND', b''))
print(f'Created {png_path}')
print('Done!')
