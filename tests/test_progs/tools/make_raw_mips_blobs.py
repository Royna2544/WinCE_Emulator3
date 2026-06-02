#!/usr/bin/env python3
from pathlib import Path
import struct

OUT = Path(__file__).resolve().parent.parent / "raw_blobs"
OUT.mkdir(parents=True, exist_ok=True)

def emit(name, words):
    path = OUT / name
    with path.open("wb") as f:
        for word in words:
            f.write(struct.pack("<I", word))
    print(path)

# ordinary delay slot: expected v0 = 1
emit("delay_slot_returns_1.bin", [
    0x24020000, # addiu v0, zero, 0
    0x10000001, # beq zero, zero, +1
    0x24020001, # addiu v0, zero, 1
    0x03e00008, # jr ra
    0x00000000, # nop
])

# bgezl taken: expected v0 = 2
emit("bgezl_taken_returns_2.bin", [
    0x24020000, # addiu v0, zero, 0
    0x04030001, # bgezl zero, +1
    0x24020002, # addiu v0, zero, 2
    0x03e00008, # jr ra
    0x00000000, # nop
])

# bltzl not taken: expected v0 = 5, delay slot annulled.
emit("bltzl_not_taken_returns_5.bin", [
    0x24020005, # addiu v0, zero, 5
    0x04020003, # bltzl zero, +3
    0x24020063, # addiu v0, zero, 99
    0x03e00008, # jr ra
    0x00000000, # nop
    0x24020007, # addiu v0, zero, 7
    0x03e00008, # jr ra
    0x00000000, # nop
])
