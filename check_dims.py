import struct
import os

def get_png_dimensions(file_path):
    with open(file_path, 'rb') as f:
        data = f.read(24)
        if data[:8] != b'\x89PNG\r\n\x1a\n':
            return None
        w, h = struct.unpack('>LL', data[16:24])
        return w, h

path = r"f:\projects\emiyashiro\assets\images\characters\hf_shirou_spritesheet_final_v2_1767279221195.png"
if os.path.exists(path):
    dims = get_png_dimensions(path)
    print(f"Dimensions: {dims}")
else:
    print("File not found")
