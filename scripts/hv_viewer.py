#!/usr/bin/env python3
r"""
Windows CE/NT registry hive (.hv) parser and RegEdit-style viewer.

Usage:

    # Open a RegEdit-style GUI viewer.
    python scripts/hv_viewer.py D:\INAVI_Emulator\DUMPPLZ\Windows\default.hv --root HKLM

    # Print a quick parse summary without opening the GUI.
    python scripts/hv_viewer.py default.hv --root HKLM --summary --no-gui

    # Export this repo's regs.json-compatible flattened registry JSON.
    python scripts/hv_viewer.py default.hv --root HKLM --export-json default_regs.json --no-gui

The parser is intentionally small and dependency-free. It understands the hive
records needed to browse ordinary Windows NT and dumped Windows CE hives:

- Windows NT regf/hbin/nk/vk/lf/lh/li/ri cell records.
- Windows CE compact EKIM/CEDB key/value records from dumped .hv files.
"""

from __future__ import annotations

import argparse
import binascii
import json
import os
import struct
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Dict, Iterable, List, Optional, Sequence, Tuple


REG_TYPES = {
    0: "REG_NONE",
    1: "REG_SZ",
    2: "REG_EXPAND_SZ",
    3: "REG_BINARY",
    4: "REG_DWORD",
    5: "REG_DWORD_BIG_ENDIAN",
    6: "REG_LINK",
    7: "REG_MULTI_SZ",
    8: "REG_RESOURCE_LIST",
    9: "REG_FULL_RESOURCE_DESCRIPTOR",
    10: "REG_RESOURCE_REQUIREMENTS_LIST",
    11: "REG_QWORD",
}

JSON_TYPE_NAMES = {
    1: "REG_SZ",
    2: "REG_EXPAND_SZ",
    3: "REG_BINARY",
    4: "REG_DWORD",
    7: "REG_MULTI_SZ",
}

REG_NONE      = 0
REG_SZ        = 1
REG_EXPAND_SZ = 2
REG_BINARY    = 3
REG_DWORD     = 4
REG_MULTI_SZ  = 7

ROOT_ALIASES = {
    "HKCR": "hkcr",
    "HKEY_CLASSES_ROOT": "hkcr",
    "HKCU": "hkcu",
    "HKEY_CURRENT_USER": "hkcu",
    "HKLM": "hklm",
    "HKEY_LOCAL_MACHINE": "hklm",
    "HKU": "hku",
    "HKEY_USERS": "hku",
}


class HiveParseError(Exception):
    pass


@dataclass
class HiveValue:
    name: str
    value_type: int
    data: bytes
    # Write-support metadata populated by the parser; -1/0 means "not tracked".
    file_offset: int = field(default=-1, repr=False)      # abs byte offset of record/cell payload
    ce_name_chars: int = field(default=0, repr=False)     # CEDB: UTF-16 name char count
    ce_total_size: int = field(default=0, repr=False)     # CEDB: aligned record size in file
    regf_inline: bool = field(default=False, repr=False)  # regf: data packed into vk.DataOffset
    regf_data_payload: int = field(default=-1, repr=False)# regf: abs offset of data cell payload

    @property
    def type_name(self) -> str:
        return REG_TYPES.get(self.value_type, f"REG_TYPE_{self.value_type}")

    def display_data(self) -> str:
        if self.value_type in (1, 2, 6):
            return decode_utf16z(self.data)
        if self.value_type == 7:
            return "; ".join(decode_multi_sz(self.data))
        if self.value_type == 4 and len(self.data) >= 4:
            value = struct.unpack_from("<I", self.data)[0]
            return f"0x{value:08x} ({value})"
        if self.value_type == 5 and len(self.data) >= 4:
            value = struct.unpack_from(">I", self.data)[0]
            return f"0x{value:08x} ({value})"
        if self.value_type == 11 and len(self.data) >= 8:
            value = struct.unpack_from("<Q", self.data)[0]
            return f"0x{value:016x} ({value})"
        if not self.data:
            return ""
        head = " ".join(f"{byte:02x}" for byte in self.data[:64])
        if len(self.data) > 64:
            head += f" ... ({len(self.data)} bytes)"
        return head

    def to_json_value(self) -> Dict[str, object]:
        ty = JSON_TYPE_NAMES.get(self.value_type, "REG_BINARY")
        if self.value_type in (1, 2):
            data: object = decode_utf16z(self.data)
        elif self.value_type == 7:
            data = decode_multi_sz(self.data)
        elif self.value_type == 4 and len(self.data) >= 4:
            data = struct.unpack_from("<I", self.data)[0]
        else:
            data = list(self.data)
        return {"type": ty, "data": data}


@dataclass
class HiveKey:
    name: str
    offset: int          # CEDB: abs file offset of key record; regf: cell offset
    parent_offset: int   # CEDB: parent OID; regf: parent cell offset
    last_write: Optional[datetime]
    values: List[HiveValue] = field(default_factory=list)
    children: List["HiveKey"] = field(default_factory=list)
    # CEDB write-support (populated by parser; 0 for root / untracked)
    ce_oid: int = field(default=0, repr=False)
    ce_last_child_oid: int = field(default=0, repr=False)
    ce_last_value_oid: int = field(default=0, repr=False)


class RegistryHive:
    def __init__(self, path: str, root_name: str = "HKLM") -> None:
        self.path = path
        self.root_name = normalize_root(root_name)
        with open(path, "rb") as f:
            self.data = bytearray(f.read())
        self.dirty = False
        self._ce_max_oid = 0
        self._undo_stack: List[bytearray] = []
        self._redo_stack: List[bytearray] = []
        self._data_at_save: bytearray = bytearray(self.data)
        self.format_name = "unknown"
        self.root_offset = 0
        self.hbins_size = 0
        self.filename = ""
        self._key_cache: Dict[int, HiveKey] = {}
        if self.data[:4] == b"regf":
            self.format_name = "regf"
            self._parse_header()
            self.root = self._parse_key(self.root_offset, [])
            self.root.name = self.root_name
        elif len(self.data) >= 0x24 and self.data[8:12] == b"EKIM":
            self.format_name = "ce_cedb"
            self.hbins_size = self._u32_abs(0x20)
            self.filename = "Windows CE compact CEDB hive"
            self.root = self._parse_ce_cedb_hive()
        else:
            raise HiveParseError("missing regf or Windows CE EKIM hive signature")

    def _parse_header(self) -> None:
        if len(self.data) < 0x1000:
            raise HiveParseError("file is smaller than a registry hive base block")
        if self.data[:4] != b"regf":
            raise HiveParseError("missing regf hive signature")
        self.root_offset = self._u32_abs(0x24)
        self.hbins_size = self._u32_abs(0x28)
        raw_name = self.data[0x30:0x70]
        self.filename = raw_name.decode("utf-16le", errors="ignore").rstrip("\x00")
        if self._cell_payload_offset(self.root_offset) + 2 > len(self.data):
            raise HiveParseError(f"root cell offset 0x{self.root_offset:x} is outside the file")

    def _parse_key(self, cell_offset: int, stack: Sequence[int]) -> HiveKey:
        if cell_offset in self._key_cache:
            return self._key_cache[cell_offset]
        if cell_offset in stack:
            raise HiveParseError(f"cycle in key tree at cell 0x{cell_offset:x}")

        payload = self._cell_payload_offset(cell_offset)
        sig = self.data[payload:payload + 2]
        if sig != b"nk":
            raise HiveParseError(f"cell 0x{cell_offset:x} is {sig!r}, expected nk")

        flags = self._u16(payload + 0x02)
        last_write = filetime_to_datetime(self._u64(payload + 0x04))
        parent = self._u32(payload + 0x10)
        stable_count = self._u32(payload + 0x14)
        volatile_count = self._u32(payload + 0x18)
        stable_list = self._u32(payload + 0x1C)
        volatile_list = self._u32(payload + 0x20)
        value_count = self._u32(payload + 0x24)
        value_list = self._u32(payload + 0x28)
        name_len = self._u16(payload + 0x48)
        name_raw = self._bytes(payload + 0x4C, name_len)
        name = decode_name(name_raw, compressed=bool(flags & 0x20))

        key = HiveKey(name=name, offset=cell_offset, parent_offset=parent, last_write=last_write)
        self._key_cache[cell_offset] = key

        key.values = self._parse_values(value_list, value_count)
        child_offsets: List[int] = []
        if stable_count and stable_list != 0xFFFFFFFF:
            child_offsets.extend(self._parse_subkey_index(stable_list))
        if volatile_count and volatile_list != 0xFFFFFFFF:
            child_offsets.extend(self._parse_subkey_index(volatile_list))
        for child_offset in child_offsets:
            try:
                key.children.append(self._parse_key(child_offset, [*stack, cell_offset]))
            except HiveParseError as exc:
                key.children.append(
                    HiveKey(
                        name=f"<parse error at 0x{child_offset:x}: {exc}>",
                        offset=child_offset,
                        parent_offset=cell_offset,
                        last_write=None,
                    )
                )
        key.children.sort(key=lambda child: child.name.lower())
        key.values.sort(key=lambda value: value.name.lower())
        return key

    def _parse_values(self, list_offset: int, count: int) -> List[HiveValue]:
        if count == 0 or list_offset == 0xFFFFFFFF:
            return []
        payload = self._cell_payload_offset(list_offset)
        values: List[HiveValue] = []
        for index in range(count):
            value_offset = self._u32(payload + index * 4)
            if value_offset == 0xFFFFFFFF:
                continue
            try:
                values.append(self._parse_value(value_offset))
            except HiveParseError as exc:
                values.append(HiveValue(f"<parse error at 0x{value_offset:x}>", 3, str(exc).encode()))
        return values

    def _parse_value(self, cell_offset: int) -> HiveValue:
        payload = self._cell_payload_offset(cell_offset)
        sig = self.data[payload:payload + 2]
        if sig != b"vk":
            raise HiveParseError(f"cell 0x{cell_offset:x} is {sig!r}, expected vk")
        name_len = self._u16(payload + 0x02)
        raw_data_len = self._u32(payload + 0x04)
        data_offset = self._u32(payload + 0x08)
        value_type = self._u32(payload + 0x0C)
        flags = self._u16(payload + 0x10)
        name_raw = self._bytes(payload + 0x14, name_len)
        name = decode_name(name_raw, compressed=bool(flags & 0x01)) if name_len else "(Default)"

        data_len = raw_data_len & 0x7FFFFFFF
        inline = bool(raw_data_len & 0x80000000)
        if inline:
            data = struct.pack("<I", data_offset)[:data_len]
            data_cell_payload = -1
        elif data_len == 0 or data_offset == 0xFFFFFFFF:
            data = b""
            data_cell_payload = -1
        else:
            data_cell_payload = self._cell_payload_offset(data_offset)
            data = self._bytes(data_cell_payload, data_len)
        return HiveValue(
            name=name,
            value_type=value_type,
            data=data,
            file_offset=payload,
            regf_inline=inline,
            regf_data_payload=data_cell_payload,
        )

    def _parse_subkey_index(self, cell_offset: int) -> List[int]:
        payload = self._cell_payload_offset(cell_offset)
        sig = self.data[payload:payload + 2]
        count = self._u16(payload + 0x02)
        offsets: List[int] = []
        if sig in (b"lf", b"lh"):
            for index in range(count):
                offsets.append(self._u32(payload + 0x04 + index * 8))
        elif sig == b"li":
            for index in range(count):
                offsets.append(self._u32(payload + 0x04 + index * 4))
        elif sig == b"ri":
            for index in range(count):
                nested = self._u32(payload + 0x04 + index * 4)
                offsets.extend(self._parse_subkey_index(nested))
        else:
            raise HiveParseError(f"cell 0x{cell_offset:x} is {sig!r}, expected subkey index")
        return offsets

    def _parse_ce_cedb_hive(self) -> HiveKey:
        """Parse CE compact hive records.

        CE 5/6 compact registry hives on this target are backed by the CE object
        store/CEDB format, not the desktop regf/nk/vk cell format. The useful
        records here have a length word with a high nibble tag:

        - 0xC0000000 | byte_length: registry key record
        - 0xD0000000 | byte_length: registry value record

        Keys carry an OID and parent OID. Values in these hives appear directly
        after their owning key record, so value ownership is recovered from the
        record stream. This is an inspection/export tool; it avoids rewriting
        the hive or inventing missing relationships.
        """
        root = HiveKey(self.root_name, offset=0, parent_offset=0, last_write=None)
        records = self._scan_ce_cedb_records()
        keys: Dict[int, HiveKey] = {}
        key_records: Dict[int, Dict[str, object]] = {}
        value_records: Dict[int, Dict[str, object]] = {}

        for record in records:
            oid = int(record["oid"])
            if record["kind"] == "key":
                key_records[oid] = record
                keys[oid] = HiveKey(
                    name=str(record["name"]),
                    offset=int(record["offset"]),
                    parent_offset=0,
                    last_write=None,
                )
            elif record["kind"] == "value":
                value_records[oid] = record

        assigned_children = set()
        # Track max OID for new-record allocation
        for record in records:
            oid = int(record["oid"]) & 0x0FFFFFFF
            if oid > self._ce_max_oid:
                self._ce_max_oid = oid

        # Use key_records (one entry per OID, last-write-wins) so that duplicate
        # physical copies of the same record — left by CE CEDB journaling — do not
        # cause children or values to be appended multiple times.
        for record in key_records.values():
            oid = int(record["oid"])
            key = keys[oid]
            key.ce_oid = oid
            key.ce_last_child_oid = int(record["last_child_oid"]) & 0x0FFFFFFF
            key.ce_last_value_oid = int(record["last_value_oid"]) & 0x0FFFFFFF
            for child_oid in reversed(self._walk_ce_record_chain(int(record["last_child_oid"]), key_records)):
                child = keys.get(child_oid)
                if child is None or child is key:
                    continue
                child.parent_offset = oid
                key.children.append(child)
                assigned_children.add(child_oid)

        for record in key_records.values():
            key = keys.get(int(record["oid"]))
            if key is None:
                continue
            for value_oid in reversed(
                self._walk_ce_record_chain(int(record["last_value_oid"]), value_records)
            ):
                value_record = value_records[value_oid]
                key.values.append(
                    HiveValue(
                        name=str(value_record["name"]) or "(Default)",
                        value_type=int(value_record["value_type"]),
                        data=bytes(value_record["data"]),
                        file_offset=int(value_record["offset"]),
                        ce_name_chars=int(value_record["name_chars"]),
                        ce_total_size=int(value_record["total_size"]),
                    )
                )

        for oid, key in keys.items():
            if oid not in assigned_children:
                root.children.append(key)

        self._sort_tree(root)
        return root

    def _walk_ce_record_chain(
        self,
        start_oid: int,
        records_by_oid: Dict[int, Dict[str, object]],
    ) -> List[int]:
        chain: List[int] = []
        seen = set()
        oid = start_oid & 0x0FFFFFFF
        while oid and oid not in seen:
            seen.add(oid)
            record = records_by_oid.get(oid)
            if record is None:
                break
            chain.append(oid)
            oid = int(record["prev_oid"]) & 0x0FFFFFFF
        return chain

    def _key_from_ce_record(self, record: Dict[str, object]) -> HiveKey:
        return HiveKey(
            name=str(record["name"]),
            offset=int(record["offset"]),
            parent_offset=0,
            last_write=None,
        )

    def _value_from_ce_record(self, record: Dict[str, object]) -> HiveValue:
        return HiveValue(
            name=str(record["name"]) or "(Default)",
            value_type=int(record["value_type"]),
            data=bytes(record["data"]),
        )

    def _scan_ce_cedb_records(self) -> List[Dict[str, object]]:
        records: List[Dict[str, object]] = []
        offset = 0
        data_len = len(self.data)
        while offset + 0x20 <= data_len:
            raw = struct.unpack_from("<I", self.data, offset)[0]
            tag = raw & 0xF0000000
            length = raw & 0x0FFFFFFF
            parsed: Optional[Dict[str, object]] = None
            if tag == 0xC0000000:
                parsed = self._try_parse_ce_key_record(offset, length)
            elif tag == 0xD0000000:
                parsed = self._try_parse_ce_value_record(offset, length)

            if parsed is not None:
                records.append(parsed)
                offset += int(parsed["total_size"])
            else:
                offset += 4
        return records

    def _try_parse_ce_key_record(self, offset: int, length: int) -> Optional[Dict[str, object]]:
        if length < 0x10 or length > 0x2000 or offset + 0x20 > len(self.data):
            return None
        name_chars = self._u32(offset + 0x18)
        if name_chars == 0 or name_chars > 512:
            return None
        name_offset = offset + 0x1C
        name_bytes = name_chars * 2
        total_size = align4(0x1C + name_bytes)
        if offset + total_size > len(self.data):
            return None
        name = decode_utf16z(self._bytes(name_offset, name_bytes))
        if not is_plausible_name(name):
            return None
        return {
            "kind": "key",
            "offset": offset,
            "length": length,
            "total_size": total_size,
            "oid": self._u32(offset + 0x08) & 0x0FFFFFFF,
            "prev_oid": self._u32(offset + 0x0C) & 0x0FFFFFFF,
            "last_child_oid": self._u32(offset + 0x10) & 0x0FFFFFFF,
            "last_value_oid": self._u32(offset + 0x14) & 0x0FFFFFFF,
            "name": name,
        }

    def _try_parse_ce_value_record(self, offset: int, length: int) -> Optional[Dict[str, object]]:
        if length < 0x10 or length > 0x2000 or offset + 0x18 > len(self.data):
            return None
        value_type = self._u16(offset + 0x10)
        data_len = self._u16(offset + 0x12)
        name_chars = self._u16(offset + 0x14)
        if value_type not in REG_TYPES or name_chars > 512 or data_len > length:
            return None
        name_offset = offset + 0x16
        name_bytes = name_chars * 2
        value_offset = name_offset + name_bytes
        total_size = align4(value_offset + data_len - offset)
        if offset + total_size > len(self.data):
            return None
        name = decode_utf16z(self._bytes(name_offset, name_bytes)) if name_chars else "(Default)"
        if not is_plausible_name(name):
            return None
        return {
            "kind": "value",
            "offset": offset,
            "length": length,
            "total_size": total_size,
            "oid": self._u32(offset + 0x08) & 0x0FFFFFFF,
            "prev_oid": self._u32(offset + 0x0C) & 0x0FFFFFFF,
            "value_type": value_type,
            "name": name,
            "name_chars": name_chars,
            "data": self._bytes(value_offset, data_len),
        }

    def _sort_tree(self, key: HiveKey) -> None:
        key.children.sort(key=lambda child: child.name.lower())
        key.values.sort(key=lambda value: value.name.lower())
        for child in key.children:
            self._sort_tree(child)

    # ------------------------------------------------------------------
    # Write support
    # ------------------------------------------------------------------

    def write_value(self, value: HiveValue, new_data: bytes, new_type: int) -> None:
        """Patch *value* in-place in self.data, then mark the hive dirty.

        For CE CEDB: new data must fit within the original aligned record block
        (same align4 total_size as before).  For regf: non-inline cells must
        not exceed the original data cell size; inline cells are limited to 4 B.
        """
        if self.format_name == "ce_cedb":
            self._write_cedb_value(value, new_data, new_type)
        elif self.format_name == "regf":
            self._write_regf_value(value, new_data, new_type)
        else:
            raise ValueError(f"write not supported for format {self.format_name!r}")
        value.value_type = new_type
        value.data = bytes(new_data)
        self.dirty = True

    def _write_cedb_value(self, value: HiveValue, new_data: bytes, new_type: int) -> None:
        if value.file_offset < 0:
            raise ValueError("value has no tracked file offset")
        overhead = 0x16 + value.ce_name_chars * 2
        capacity = value.ce_total_size - overhead
        if capacity <= 0:
            raise ValueError("value record has no data capacity")
        if len(new_data) > capacity:
            raise ValueError(f"new data {len(new_data)} B exceeds in-place capacity {capacity} B")
        new_total = align4(overhead + len(new_data))
        if new_total != value.ce_total_size:
            raise ValueError(
                f"new data length would change the record block size "
                f"(align4({overhead}+{len(new_data)})={new_total} != {value.ce_total_size}); "
                f"acceptable lengths: {capacity - 3}–{capacity} bytes"
            )
        data_abs = value.file_offset + overhead
        struct.pack_into("<H", self.data, value.file_offset + 0x10, new_type)
        struct.pack_into("<H", self.data, value.file_offset + 0x12, len(new_data))
        self.data[data_abs:data_abs + capacity] = b"\x00" * capacity
        self.data[data_abs:data_abs + len(new_data)] = new_data

    def _write_regf_value(self, value: HiveValue, new_data: bytes, new_type: int) -> None:
        if value.file_offset < 0:
            raise ValueError("value has no tracked file offset")
        vk = value.file_offset
        old_raw = struct.unpack_from("<I", self.data, vk + 0x04)[0]
        old_data_len = old_raw & 0x7FFFFFFF
        if value.regf_inline:
            if len(new_data) > 4:
                raise ValueError("inline regf value limited to 4 bytes")
            packed = int.from_bytes(new_data.ljust(4, b"\x00"), "little")
            struct.pack_into("<I", self.data, vk + 0x08, packed)
            struct.pack_into("<I", self.data, vk + 0x04, 0x80000000 | len(new_data))
        else:
            if len(new_data) > old_data_len:
                raise ValueError(
                    f"new data {len(new_data)} B exceeds data cell capacity {old_data_len} B"
                )
            if value.regf_data_payload < 0:
                raise ValueError("non-inline value has no data cell payload offset")
            self.data[value.regf_data_payload:value.regf_data_payload + old_data_len] = (
                new_data + b"\x00" * (old_data_len - len(new_data))
            )
            struct.pack_into("<I", self.data, vk + 0x04, len(new_data))
        struct.pack_into("<I", self.data, vk + 0x0C, new_type)
        self._update_regf_checksum()

    def _update_regf_checksum(self) -> None:
        checksum = 0
        for i in range(0, 0x1FC, 4):
            checksum ^= struct.unpack_from("<I", self.data, i)[0]
        struct.pack_into("<I", self.data, 0x1FC, checksum)

    def save(self, path: Optional[str] = None) -> None:
        dest = path or self.path
        with open(dest, "wb") as f:
            f.write(self.data)
        self.path = dest
        self.dirty = False
        self._data_at_save = bytearray(self.data)

    def push_undo(self) -> None:
        """Snapshot current data before a mutation. Clears the redo stack."""
        if len(self._undo_stack) >= 50:
            self._undo_stack.pop(0)
        self._undo_stack.append(bytearray(self.data))
        self._redo_stack.clear()

    def undo(self) -> bool:
        if not self._undo_stack:
            return False
        self._redo_stack.append(bytearray(self.data))
        self.data = self._undo_stack.pop()
        self._reload()
        return True

    def redo(self) -> bool:
        if not self._redo_stack:
            return False
        self._undo_stack.append(bytearray(self.data))
        self.data = self._redo_stack.pop()
        self._reload()
        return True

    def _reload(self) -> None:
        """Re-derive the in-memory tree from self.data after undo/redo."""
        if self.format_name == "ce_cedb":
            self._ce_max_oid = 0
            self._key_cache.clear()
            self.root = self._parse_ce_cedb_hive()
        elif self.format_name == "regf":
            self._key_cache.clear()
            self._parse_header()
            self.root = self._parse_key(self.root_offset, [])
            self.root.name = self.root_name
        self.dirty = self.data != self._data_at_save

    # ------------------------------------------------------------------
    # CE CEDB structural edit: add / delete keys and values
    # ------------------------------------------------------------------

    @staticmethod
    def _ce_ref(oid: int) -> int:
        """Pack a non-null OID as a chain reference (0x20000000 flag observed in all hives)."""
        return (0x20000000 | (oid & 0x0FFFFFFF)) if oid else 0

    def _ce_alloc_oid(self) -> int:
        self._ce_max_oid += 1
        return self._ce_max_oid

    def _ce_append(self, record: bytes) -> int:
        """Append record bytes to self.data, update hbins_size header, return new record offset."""
        offset = len(self.data)
        self.data.extend(record)
        struct.pack_into("<I", self.data, 0x20, len(self.data))
        return offset

    def _build_ce_key_record(self, oid: int, prev_oid: int, name: str) -> bytes:
        name_enc = name.encode("utf-16le")
        nc = len(name)
        total = align4(0x1C + len(name_enc))
        length = align4(0x10 + len(name_enc))
        rec = bytearray(total)
        struct.pack_into("<I", rec, 0x00, 0xC0000000 | length)
        struct.pack_into("<I", rec, 0x04, 0)
        struct.pack_into("<I", rec, 0x08, oid)
        struct.pack_into("<I", rec, 0x0C, self._ce_ref(prev_oid))
        struct.pack_into("<I", rec, 0x10, 0)  # last_child_oid (empty)
        struct.pack_into("<I", rec, 0x14, 0)  # last_value_oid (empty)
        struct.pack_into("<I", rec, 0x18, nc)
        rec[0x1C:0x1C + len(name_enc)] = name_enc
        return bytes(rec)

    def _build_ce_value_record(
        self, oid: int, prev_oid: int, name: str, vtype: int, data: bytes
    ) -> bytes:
        if name == "(Default)":
            name = ""
        name_enc = name.encode("utf-16le")
        nc = len(name)
        total = align4(0x16 + len(name_enc) + len(data))
        length = align4(0x0C + len(name_enc) + len(data))
        rec = bytearray(total)
        struct.pack_into("<I", rec, 0x00, 0xD0000000 | length)
        struct.pack_into("<I", rec, 0x04, 0)
        struct.pack_into("<I", rec, 0x08, oid)
        struct.pack_into("<I", rec, 0x0C, self._ce_ref(prev_oid))
        struct.pack_into("<H", rec, 0x10, vtype)
        struct.pack_into("<H", rec, 0x12, len(data))
        struct.pack_into("<H", rec, 0x14, nc)
        rec[0x16:0x16 + len(name_enc)] = name_enc
        rec[0x16 + len(name_enc):0x16 + len(name_enc) + len(data)] = data
        return bytes(rec)

    def add_ce_key(self, parent_key: HiveKey, name: str) -> HiveKey:
        """Append a new child key under parent_key."""
        if self.format_name != "ce_cedb":
            raise ValueError("add_ce_key only supported for CE CEDB hives")
        if any(c.name.lower() == name.lower() for c in parent_key.children):
            raise ValueError(f"Key '{name}' already exists under '{parent_key.name}'")
        oid = self._ce_alloc_oid()
        prev_oid = parent_key.ce_last_child_oid
        record = self._build_ce_key_record(oid, prev_oid, name)
        new_offset = self._ce_append(record)
        # Update parent's last_child_oid (only if parent has a real file record)
        if parent_key.offset > 0:
            struct.pack_into("<I", self.data, parent_key.offset + 0x10, self._ce_ref(oid))
        parent_key.ce_last_child_oid = oid
        new_key = HiveKey(
            name=name,
            offset=new_offset,
            parent_offset=parent_key.ce_oid,
            last_write=None,
            ce_oid=oid,
            ce_last_child_oid=0,
            ce_last_value_oid=0,
        )
        parent_key.children.append(new_key)
        parent_key.children.sort(key=lambda c: c.name.lower())
        self.dirty = True
        return new_key

    def add_ce_value(
        self, parent_key: HiveKey, name: str, vtype: int, data: bytes
    ) -> HiveValue:
        """Append a new value under parent_key."""
        if self.format_name != "ce_cedb":
            raise ValueError("add_ce_value only supported for CE CEDB hives")
        if parent_key.offset == 0:
            raise ValueError("cannot add values directly to the synthetic root key")
        display_name = name if name else "(Default)"
        if any(v.name.lower() == display_name.lower() for v in parent_key.values):
            raise ValueError(f"Value '{display_name}' already exists under '{parent_key.name}'")
        oid = self._ce_alloc_oid()
        prev_oid = parent_key.ce_last_value_oid
        record = self._build_ce_value_record(oid, prev_oid, name, vtype, data)
        new_offset = self._ce_append(record)
        nc = len(name) if name else 0
        total = len(record)
        struct.pack_into("<I", self.data, parent_key.offset + 0x14, self._ce_ref(oid))
        parent_key.ce_last_value_oid = oid
        new_value = HiveValue(
            name=display_name,
            value_type=vtype,
            data=bytes(data),
            file_offset=new_offset,
            ce_name_chars=nc,
            ce_total_size=total,
        )
        parent_key.values.append(new_value)
        parent_key.values.sort(key=lambda v: v.name.lower())
        self.dirty = True
        return new_value

    def delete_ce_value(self, parent_key: HiveKey, value: HiveValue) -> None:
        """Remove value from parent_key, patching the OID chain and zeroing the record."""
        if self.format_name != "ce_cedb":
            raise ValueError("delete_ce_value only supported for CE CEDB hives")
        val_oid = struct.unpack_from("<I", self.data, value.file_offset + 0x08)[0] & 0x0FFFFFFF
        val_prev = struct.unpack_from("<I", self.data, value.file_offset + 0x0C)[0]
        if parent_key.ce_last_value_oid == val_oid:
            struct.pack_into("<I", self.data, parent_key.offset + 0x14, val_prev)
            parent_key.ce_last_value_oid = val_prev & 0x0FFFFFFF
        else:
            for sib in parent_key.values:
                if sib is value or sib.file_offset < 0:
                    continue
                sib_prev_raw = struct.unpack_from("<I", self.data, sib.file_offset + 0x0C)[0]
                if (sib_prev_raw & 0x0FFFFFFF) == val_oid:
                    struct.pack_into("<I", self.data, sib.file_offset + 0x0C, val_prev)
                    break
        self.data[value.file_offset:value.file_offset + value.ce_total_size] = (
            b"\x00" * value.ce_total_size
        )
        parent_key.values.remove(value)
        self.dirty = True

    def delete_ce_key(self, key: HiveKey, parent_key: HiveKey) -> None:
        """Recursively delete key and all descendants, patching parent chain."""
        if self.format_name != "ce_cedb":
            raise ValueError("delete_ce_key only supported for CE CEDB hives")
        for child in list(key.children):
            self.delete_ce_key(child, key)
        for value in list(key.values):
            self.delete_ce_value(key, value)
        key_oid = struct.unpack_from("<I", self.data, key.offset + 0x08)[0] & 0x0FFFFFFF
        key_prev = struct.unpack_from("<I", self.data, key.offset + 0x0C)[0]
        if parent_key.ce_last_child_oid == key_oid:
            if parent_key.offset > 0:
                struct.pack_into("<I", self.data, parent_key.offset + 0x10, key_prev)
            parent_key.ce_last_child_oid = key_prev & 0x0FFFFFFF
        else:
            for sib in parent_key.children:
                if sib is key or sib.offset <= 0:
                    continue
                sib_prev_raw = struct.unpack_from("<I", self.data, sib.offset + 0x0C)[0]
                if (sib_prev_raw & 0x0FFFFFFF) == key_oid:
                    struct.pack_into("<I", self.data, sib.offset + 0x0C, key_prev)
                    break
        nc = struct.unpack_from("<I", self.data, key.offset + 0x18)[0]
        key_total = align4(0x1C + nc * 2)
        self.data[key.offset:key.offset + key_total] = b"\x00" * key_total
        parent_key.children.remove(key)
        self.dirty = True

    def iter_keys(self) -> Iterable[Tuple[str, HiveKey]]:
        stack: List[Tuple[str, HiveKey]] = [(self.root.name.lower(), self.root)]
        while stack:
            path, key = stack.pop()
            yield path, key
            for child in reversed(key.children):
                stack.append((f"{path}\\{child.name.lower()}", child))

    def to_regs_json(self) -> Dict[str, object]:
        keys: Dict[str, object] = {}
        for path, key in self.iter_keys():
            keys[path] = {
                "values": {
                    normalize_value_json_name(value.name): value.to_json_value()
                    for value in key.values
                }
            }
        return {
            "version": 1,
            "source": f"Parsed from {os.path.abspath(self.path)}",
            "keys": keys,
        }

    def _cell_payload_offset(self, cell_offset: int) -> int:
        if cell_offset == 0xFFFFFFFF:
            raise HiveParseError("null hive cell offset")
        cell_header = 0x1000 + cell_offset
        if cell_header < 0 or cell_header + 4 > len(self.data):
            raise HiveParseError(f"cell offset 0x{cell_offset:x} is outside the file")
        return cell_header + 4

    def _bytes(self, offset: int, length: int) -> bytes:
        if offset < 0 or offset + length > len(self.data):
            raise HiveParseError(f"read outside hive at file offset 0x{offset:x}, length 0x{length:x}")
        return self.data[offset:offset + length]

    def _u16(self, offset: int) -> int:
        return struct.unpack_from("<H", self._bytes(offset, 2))[0]

    def _u32(self, offset: int) -> int:
        return struct.unpack_from("<I", self._bytes(offset, 4))[0]

    def _u64(self, offset: int) -> int:
        return struct.unpack_from("<Q", self._bytes(offset, 8))[0]

    def _u32_abs(self, offset: int) -> int:
        return struct.unpack_from("<I", self.data, offset)[0]


def normalize_root(root: str) -> str:
    root = root.strip()
    if not root:
        return "hklm"
    return ROOT_ALIASES.get(root.upper(), root.lower())


def normalize_value_json_name(name: str) -> str:
    return "default" if name == "(Default)" else name.lower()


def decode_name(raw: bytes, compressed: bool) -> str:
    if compressed:
        return raw.decode("latin-1", errors="replace")
    return raw.decode("utf-16le", errors="replace").rstrip("\x00")


def decode_utf16z(raw: bytes) -> str:
    if not raw:
        return ""
    if len(raw) % 2:
        raw += b"\x00"
    return raw.decode("utf-16le", errors="replace").rstrip("\x00")


def decode_multi_sz(raw: bytes) -> List[str]:
    text = decode_utf16z(raw)
    return [part for part in text.split("\x00") if part]


def align4(value: int) -> int:
    return (value + 3) & ~3


def is_plausible_name(name: str) -> bool:
    if not name:
        return True
    if any(ord(ch) < 0x20 and ch not in "\t" for ch in name):
        return False
    return sum(1 for ch in name if ch == "\ufffd") == 0


def filetime_to_datetime(value: int) -> Optional[datetime]:
    if value == 0:
        return None
    try:
        unix_us = (value - 116444736000000000) // 10
        return datetime.fromtimestamp(unix_us / 1_000_000, tz=timezone.utc)
    except (OSError, ValueError, OverflowError):
        return None


class HiveViewer:
    def __init__(self, hive: RegistryHive) -> None:
        import tkinter as tk
        from tkinter import filedialog, ttk

        self.tk = tk
        self.ttk = ttk
        self.filedialog = filedialog
        self.hive = hive
        self.item_to_key: Dict[str, HiveKey] = {}
        self.item_to_value: Dict[str, HiveValue] = {}

        self.root = tk.Tk()
        self.root.title(f"Registry Hive Viewer - {os.path.basename(hive.path)}")
        self.root.geometry("1100x700")
        self.root.minsize(720, 420)

        self._build_menu()
        self._build_body()
        self._populate_tree()

    def run(self) -> None:
        self.root.protocol("WM_DELETE_WINDOW", self._on_close)
        self.root.mainloop()

    def _build_menu(self) -> None:
        menu = self.tk.Menu(self.root)

        file_menu = self.tk.Menu(menu, tearoff=False)
        file_menu.add_command(label="Open Hive...", command=self._open_hive)
        file_menu.add_command(label="Export JSON...", command=self._export_json)
        file_menu.add_separator()
        file_menu.add_command(label="Save", command=self._save, accelerator="Ctrl+S")
        file_menu.add_command(label="Save As...", command=self._save_as)
        file_menu.add_separator()
        file_menu.add_command(label="Exit", command=self._on_close)
        menu.add_cascade(label="File", menu=file_menu)

        edit_menu = self.tk.Menu(menu, tearoff=False)
        edit_menu.add_command(label="Undo", command=self._undo, accelerator="Ctrl+Z")
        edit_menu.add_command(label="Redo", command=self._redo, accelerator="Ctrl+Y")
        menu.add_cascade(label="Edit", menu=edit_menu)

        self.root.config(menu=menu)
        self.root.bind("<Control-s>", lambda _e: self._save())
        self.root.bind("<Control-z>", lambda _e: self._undo())
        self.root.bind("<Control-y>", lambda _e: self._redo())

    def _build_body(self) -> None:
        outer = self.ttk.Frame(self.root)
        outer.pack(fill="both", expand=True)

        self.path_var = self.tk.StringVar(value=self.hive.root.name)
        path_bar = self.ttk.Entry(outer, textvariable=self.path_var, state="readonly")
        path_bar.pack(fill="x", padx=6, pady=(6, 4))

        paned = self.ttk.PanedWindow(outer, orient="horizontal")
        paned.pack(fill="both", expand=True, padx=6, pady=0)

        left = self.ttk.Frame(paned)
        right = self.ttk.Frame(paned)
        paned.add(left, weight=1)
        paned.add(right, weight=3)

        self.tree = self.ttk.Treeview(left, show="tree")
        tree_scroll = self.ttk.Scrollbar(left, orient="vertical", command=self.tree.yview)
        self.tree.configure(yscrollcommand=tree_scroll.set)
        self.tree.pack(side="left", fill="both", expand=True)
        tree_scroll.pack(side="right", fill="y")
        self.tree.bind("<<TreeviewSelect>>", self._on_select)

        columns = ("name", "type", "data")
        self.values = self.ttk.Treeview(right, columns=columns, show="headings")
        self.values.heading("name", text="Name")
        self.values.heading("type", text="Type")
        self.values.heading("data", text="Data")
        self.values.column("name", width=220, minwidth=120, stretch=False)
        self.values.column("type", width=150, minwidth=110, stretch=False)
        self.values.column("data", width=520, minwidth=160, stretch=True)
        value_scroll = self.ttk.Scrollbar(right, orient="vertical", command=self.values.yview)
        self.values.configure(yscrollcommand=value_scroll.set)
        self.values.pack(side="left", fill="both", expand=True)
        value_scroll.pack(side="right", fill="y")
        self.values.bind("<Double-1>", self._on_value_double_click)

        self.status_var = self.tk.StringVar(value="")
        status = self.ttk.Label(outer, textvariable=self.status_var, anchor="w")
        status.pack(fill="x", padx=6, pady=(4, 6))

        self._bind_context_menus()

    def _populate_tree(self) -> None:
        self.tree.delete(*self.tree.get_children())
        self.item_to_key.clear()
        root_item = self.tree.insert("", "end", text=self.hive.root.name, open=True)
        self.item_to_key[root_item] = self.hive.root
        self._insert_children(root_item, self.hive.root)
        self.tree.selection_set(root_item)
        self._show_key(self.hive.root)

    def _insert_children(self, parent_item: str, key: HiveKey) -> None:
        for child in key.children:
            item = self.tree.insert(parent_item, "end", text=child.name, open=False)
            self.item_to_key[item] = child
            if child.children:
                self._insert_children(item, child)

    def _on_select(self, _event: object) -> None:
        selection = self.tree.selection()
        if not selection:
            return
        key = self.item_to_key.get(selection[0])
        if key is not None:
            self._show_key(key)

    def _show_key(self, key: HiveKey) -> None:
        self.values.delete(*self.values.get_children())
        self.item_to_value.clear()
        for value in key.values:
            iid = self.values.insert("", "end", values=(value.name, value.type_name, value.display_data()))
            self.item_to_value[iid] = value
        path = self._selected_path()
        self.path_var.set(path)
        timestamp = key.last_write.isoformat() if key.last_write else "unknown time"
        self.status_var.set(
            f"{path}    {len(key.children)} subkeys, {len(key.values)} values, "
            f"cell 0x{key.offset:x}, last write {timestamp}"
        )

    def _selected_path(self) -> str:
        selection = self.tree.selection()
        if not selection:
            return self.hive.root.name
        parts = []
        item = selection[0]
        while item:
            parts.append(self.tree.item(item, "text"))
            item = self.tree.parent(item)
        return "\\".join(reversed(parts))

    def _open_hive(self) -> None:
        path = self.filedialog.askopenfilename(
            title="Open registry hive",
            filetypes=(("Registry hives", "*.hv *.dat *.*"), ("All files", "*.*")),
        )
        if not path:
            return
        try:
            self.hive = RegistryHive(path, self.hive.root_name)
            self.root.title(f"Registry Hive Viewer - {os.path.basename(path)}")
            self._populate_tree()
        except Exception as exc:  # GUI boundary: show any parser problem.
            from tkinter import messagebox

            messagebox.showerror("Hive parse error", str(exc))

    def _export_json(self) -> None:
        path = self.filedialog.asksaveasfilename(
            title="Export registry JSON",
            defaultextension=".json",
            filetypes=(("JSON", "*.json"), ("All files", "*.*")),
        )
        if not path:
            return
        with open(path, "w", encoding="utf-8") as f:
            json.dump(self.hive.to_regs_json(), f, indent=2, ensure_ascii=False)
            f.write("\n")
        self.status_var.set(f"Exported {path}")

    # ------------------------------------------------------------------
    # Write support
    # ------------------------------------------------------------------

    def _on_value_double_click(self, _event: object) -> None:
        sel = self.values.selection()
        if not sel:
            return
        value = self.item_to_value.get(sel[0])
        if value is not None:
            self._edit_value(value, sel[0])

    def _edit_value(self, value: HiveValue, item_id: str) -> None:
        if value.file_offset < 0:
            from tkinter import messagebox
            messagebox.showwarning(
                "Read-only",
                "This value has no tracked file offset and cannot be edited.",
                parent=self.root,
            )
            return

        tk = self.tk
        ttk = self.ttk

        dlg = tk.Toplevel(self.root)
        dlg.title(f"Edit Value: {value.name}")
        dlg.resizable(False, False)
        dlg.transient(self.root)
        dlg.grab_set()

        ttk.Label(dlg, text=f"Name:  {value.name}", anchor="w").grid(
            row=0, column=0, columnspan=2, sticky="w", padx=10, pady=(10, 2)
        )
        ttk.Label(dlg, text=f"Type:  {value.type_name}", anchor="w").grid(
            row=1, column=0, columnspan=2, sticky="w", padx=10, pady=2
        )

        if self.hive.format_name == "ce_cedb":
            overhead = 0x16 + value.ce_name_chars * 2
            capacity = value.ce_total_size - overhead
            cap_label = f"Capacity: {capacity} bytes  (need align4({overhead}+len)=={value.ce_total_size})"
        else:
            capacity = None
            cap_label = ""
        if cap_label:
            ttk.Label(dlg, text=cap_label, foreground="gray", anchor="w").grid(
                row=2, column=0, columnspan=2, sticky="w", padx=10, pady=2
            )

        frame = ttk.LabelFrame(dlg, text="Data")
        frame.grid(row=3, column=0, columnspan=2, padx=10, pady=6, sticky="ew")
        dlg.columnconfigure(0, weight=1)

        get_data = None  # populated below per-type

        if value.value_type == 4 and len(value.data) >= 4:  # REG_DWORD
            val_int = struct.unpack_from("<I", value.data)[0]
            hex_var = tk.StringVar(value=f"0x{val_int:08x}")
            dec_var = tk.StringVar(value=str(val_int))

            ttk.Label(frame, text="Hex:").grid(row=0, column=0, padx=6, pady=4, sticky="e")
            hex_entry = ttk.Entry(frame, textvariable=hex_var, width=16)
            hex_entry.grid(row=0, column=1, padx=6, pady=4)
            ttk.Label(frame, text="Dec:").grid(row=1, column=0, padx=6, pady=4, sticky="e")
            dec_entry = ttk.Entry(frame, textvariable=dec_var, width=16)
            dec_entry.grid(row=1, column=1, padx=6, pady=4)

            _updating = [False]

            def on_hex(*_):
                if _updating[0]:
                    return
                try:
                    s = hex_var.get().strip()
                    v = int(s, 16) if s.lower().startswith("0x") else int(s, 16)
                    _updating[0] = True
                    dec_var.set(str(v & 0xFFFFFFFF))
                    _updating[0] = False
                except ValueError:
                    pass

            def on_dec(*_):
                if _updating[0]:
                    return
                try:
                    v = int(dec_var.get().strip())
                    _updating[0] = True
                    hex_var.set(f"0x{v & 0xFFFFFFFF:08x}")
                    _updating[0] = False
                except ValueError:
                    pass

            hex_var.trace_add("write", on_hex)
            dec_var.trace_add("write", on_dec)
            hex_entry.focus_set()
            hex_entry.selection_range(0, "end")

            def get_data():  # type: ignore[misc]
                try:
                    s = hex_var.get().strip()
                    v = int(s, 16) if s.lower().startswith("0x") else int(s, 10)
                    return struct.pack("<I", v & 0xFFFFFFFF), 4
                except ValueError:
                    return None, None

        elif value.value_type in (1, 2):  # REG_SZ / REG_EXPAND_SZ
            text = decode_utf16z(value.data)
            text_var = tk.StringVar(value=text)
            entry = ttk.Entry(frame, textvariable=text_var, width=52)
            entry.pack(fill="x", padx=6, pady=6)
            entry.focus_set()
            entry.selection_range(0, "end")

            vtype = value.value_type

            def get_data():  # type: ignore[misc]
                encoded = (text_var.get() + "\x00").encode("utf-16le")
                return encoded, vtype

        elif value.value_type == 7:  # REG_MULTI_SZ
            items = decode_multi_sz(value.data)
            text_box = tk.Text(frame, height=5, width=52, font=("Courier New", 10))
            text_box.insert("1.0", "\n".join(items))
            text_box.pack(fill="both", expand=True, padx=6, pady=6)
            text_box.focus_set()

            def get_data():  # type: ignore[misc]
                lines = text_box.get("1.0", "end-1c").split("\n")
                encoded = ("\x00".join(lines) + "\x00\x00").encode("utf-16le")
                return encoded, 7

        else:  # REG_BINARY and everything else: hex editor
            hex_text = " ".join(f"{b:02x}" for b in value.data)
            text_box = tk.Text(frame, height=4, width=52, font=("Courier New", 10))
            text_box.insert("1.0", hex_text)
            text_box.pack(fill="both", expand=True, padx=6, pady=6)
            text_box.focus_set()

            vtype = value.value_type

            def get_data():  # type: ignore[misc]
                try:
                    raw = bytes.fromhex(text_box.get("1.0", "end-1c").strip().replace(" ", ""))
                    return raw, vtype
                except ValueError:
                    return None, None

        btn_frame = ttk.Frame(dlg)
        btn_frame.grid(row=4, column=0, columnspan=2, padx=10, pady=(0, 10), sticky="e")

        def on_ok() -> None:
            new_data, new_type = get_data()  # type: ignore[misc]
            if new_data is None:
                from tkinter import messagebox
                messagebox.showerror("Invalid data", "Could not parse the entered data.", parent=dlg)
                return
            self.hive.push_undo()
            try:
                self.hive.write_value(value, new_data, new_type)
            except ValueError as exc:
                self.hive._undo_stack.pop()  # roll back the snapshot we just pushed
                from tkinter import messagebox
                messagebox.showerror("Write error", str(exc), parent=dlg)
                return
            self.values.item(item_id, values=(value.name, value.type_name, value.display_data()))
            self._update_dirty_title()
            dlg.destroy()

        ttk.Button(btn_frame, text="OK", command=on_ok, width=8).pack(side="right", padx=(4, 0))
        ttk.Button(btn_frame, text="Cancel", command=dlg.destroy, width=8).pack(side="right")
        dlg.bind("<Return>", lambda _e: on_ok())
        dlg.bind("<Escape>", lambda _e: dlg.destroy())

    def _update_dirty_title(self) -> None:
        marker = " *" if self.hive.dirty else ""
        self.root.title(f"Registry Hive Viewer - {os.path.basename(self.hive.path)}{marker}")

    def _undo(self) -> None:
        saved_path = self._selected_path()
        if not self.hive.undo():
            return
        self._populate_tree()
        self._try_restore_path(saved_path)
        self._update_dirty_title()

    def _redo(self) -> None:
        saved_path = self._selected_path()
        if not self.hive.redo():
            return
        self._populate_tree()
        self._try_restore_path(saved_path)
        self._update_dirty_title()

    def _try_restore_path(self, path: str) -> None:
        """After a full tree rebuild, try to re-select the same registry path."""
        if not path:
            return
        parts = [p for p in path.split("\\") if p]
        item = ""
        for part in parts:
            found = None
            for child in self.tree.get_children(item):
                if self.tree.item(child, "text").lower() == part.lower():
                    found = child
                    break
            if found is None:
                break
            item = found
        if item:
            self.tree.see(item)
            self.tree.selection_set(item)
            key = self.item_to_key.get(item)
            if key:
                self._show_key(key)

    def _save(self) -> None:
        try:
            self.hive.save()
            self._update_dirty_title()
            self.status_var.set(f"Saved {self.hive.path}")
        except Exception as exc:
            from tkinter import messagebox
            messagebox.showerror("Save error", str(exc), parent=self.root)

    def _save_as(self) -> None:
        path = self.filedialog.asksaveasfilename(
            title="Save hive as",
            defaultextension=".hv",
            filetypes=(("Registry hives", "*.hv *.dat *.*"), ("All files", "*.*")),
        )
        if not path:
            return
        try:
            self.hive.save(path)
            self._update_dirty_title()
            self.status_var.set(f"Saved {path}")
        except Exception as exc:
            from tkinter import messagebox
            messagebox.showerror("Save error", str(exc), parent=self.root)

    # ------------------------------------------------------------------
    # Context menus: key tree (left pane) and values pane (right pane)
    # ------------------------------------------------------------------

    def _bind_context_menus(self) -> None:
        self.tree.bind("<Button-3>", self._tree_context_menu)
        self.tree.bind("<Delete>", self._on_delete_key_press)
        self.values.bind("<Button-3>", self._values_context_menu)
        self.values.bind("<Delete>", self._on_delete_value_press)

    def _tree_context_menu(self, event: object) -> None:
        tk = self.tk
        iid = self.tree.identify_row(event.y)
        if iid:
            self.tree.selection_set(iid)
        key = self.item_to_key.get(iid) if iid else None

        menu = tk.Menu(self.root, tearoff=False)
        new_sub = tk.Menu(menu, tearoff=False)
        new_sub.add_command(label="Key", command=lambda: self._new_key(iid, key))
        new_sub.add_separator()
        new_sub.add_command(label="String Value",     command=lambda: self._new_value(key, REG_SZ,        b"\x00\x00"))
        new_sub.add_command(label="DWORD Value",      command=lambda: self._new_value(key, REG_DWORD,     b"\x00\x00\x00\x00"))
        new_sub.add_command(label="Binary Value",     command=lambda: self._new_value(key, REG_BINARY,    b""))
        new_sub.add_command(label="Multi-String Value", command=lambda: self._new_value(key, REG_MULTI_SZ, b"\x00\x00\x00\x00"))
        menu.add_cascade(label="New", menu=new_sub)
        menu.add_separator()
        menu.add_command(
            label="Delete Key",
            command=lambda: self._do_delete_key(iid, key),
            state="normal" if (key and key is not self.hive.root) else "disabled",
        )
        menu.post(event.x_root, event.y_root)

    def _values_context_menu(self, event: object) -> None:
        tk = self.tk
        sel_tree = self.tree.selection()
        key = self.item_to_key.get(sel_tree[0]) if sel_tree else None
        iid = self.values.identify_row(event.y)
        if iid:
            self.values.selection_set(iid)
        value = self.item_to_value.get(iid) if iid else None

        menu = tk.Menu(self.root, tearoff=False)
        new_sub = tk.Menu(menu, tearoff=False)
        new_sub.add_command(label="String Value",     command=lambda: self._new_value(key, REG_SZ,        b"\x00\x00"))
        new_sub.add_command(label="DWORD Value",      command=lambda: self._new_value(key, REG_DWORD,     b"\x00\x00\x00\x00"))
        new_sub.add_command(label="Binary Value",     command=lambda: self._new_value(key, REG_BINARY,    b""))
        new_sub.add_command(label="Multi-String Value", command=lambda: self._new_value(key, REG_MULTI_SZ, b"\x00\x00\x00\x00"))
        menu.add_cascade(label="New", menu=new_sub)
        menu.add_separator()
        menu.add_command(
            label="Delete Value",
            command=lambda: self._do_delete_value(iid, key, value),
            state="normal" if value is not None else "disabled",
        )
        if value is not None:
            menu.add_command(label="Modify...", command=lambda: self._edit_value(value, iid))
        menu.post(event.x_root, event.y_root)

    def _on_delete_key_press(self, _event: object) -> None:
        sel = self.tree.selection()
        if not sel:
            return
        iid = sel[0]
        key = self.item_to_key.get(iid)
        if key and key is not self.hive.root:
            self._do_delete_key(iid, key)

    def _on_delete_value_press(self, _event: object) -> None:
        sel = self.values.selection()
        if not sel:
            return
        iid = sel[0]
        value = self.item_to_value.get(iid)
        sel_tree = self.tree.selection()
        key = self.item_to_key.get(sel_tree[0]) if sel_tree else None
        if value is not None and key is not None:
            self._do_delete_value(iid, key, value)

    # ------------------------------------------------------------------
    # New-key / new-value dialogs
    # ------------------------------------------------------------------

    def _new_key(self, parent_item: str, parent_key: Optional[HiveKey]) -> None:
        if self.hive.format_name != "ce_cedb":
            self._show_ce_only()
            return
        if parent_key is None:
            return
        name = self._ask_name("New Key", "Key name:")
        if name is None:
            return
        self.hive.push_undo()
        try:
            new_key = self.hive.add_ce_key(parent_key, name)
        except Exception as exc:
            self.hive._undo_stack.pop()
            from tkinter import messagebox
            messagebox.showerror("Error", str(exc), parent=self.root)
            return
        new_item = self.tree.insert(parent_item, "end", text=name, open=False)
        self.item_to_key[new_item] = new_key
        # Move to alphabetically correct position
        sorted_idx = next(i for i, c in enumerate(parent_key.children) if c is new_key)
        self.tree.move(new_item, parent_item, sorted_idx)
        self.tree.see(new_item)
        self.tree.selection_set(new_item)
        self._show_key(new_key)
        self._update_dirty_title()

    def _new_value(self, key: Optional[HiveKey], vtype: int, _unused_default: bytes = b"") -> None:
        if self.hive.format_name != "ce_cedb":
            self._show_ce_only()
            return
        if key is None or key is self.hive.root:
            from tkinter import messagebox
            messagebox.showwarning("No key selected", "Select a key first.", parent=self.root)
            return
        result = self._new_value_dialog(vtype)
        if result is None:
            return
        name, final_data, final_type = result
        self.hive.push_undo()
        try:
            new_val = self.hive.add_ce_value(key, name, final_type, final_data)
        except Exception as exc:
            self.hive._undo_stack.pop()
            from tkinter import messagebox
            messagebox.showerror("Error", str(exc), parent=self.root)
            return
        # Rebuild values pane so order matches the sorted data model
        self._show_key(key)
        for iid, v in self.item_to_value.items():
            if v is new_val:
                self.values.selection_set(iid)
                self.values.see(iid)
                break
        self._update_dirty_title()

    def _new_value_dialog(self, vtype: int) -> Optional[Tuple[str, bytes, int]]:
        """Single dialog: name + initial data for a new registry value.
        Returns (name, data_bytes, vtype) or None if cancelled."""
        tk = self.tk
        ttk = self.ttk
        result: list = [None]

        type_name = REG_TYPES.get(vtype, f"REG_UNKNOWN_{vtype}")
        dlg = tk.Toplevel(self.root)
        dlg.title(f"New {type_name} Value")
        dlg.resizable(False, False)
        dlg.transient(self.root)
        dlg.grab_set()

        ttk.Label(dlg, text=f"Type:  {type_name}", anchor="w").grid(
            row=0, column=0, columnspan=2, sticky="w", padx=10, pady=(10, 2)
        )
        ttk.Label(dlg, text="Name:", anchor="w").grid(row=1, column=0, sticky="w", padx=10, pady=2)
        name_var = tk.StringVar()
        name_entry = ttk.Entry(dlg, textvariable=name_var, width=40)
        name_entry.grid(row=1, column=1, padx=10, pady=2, sticky="ew")
        name_entry.focus()

        frame = ttk.LabelFrame(dlg, text="Initial Data")
        frame.grid(row=2, column=0, columnspan=2, padx=10, pady=6, sticky="ew")
        dlg.columnconfigure(1, weight=1)

        get_data_fn = None

        if vtype == REG_DWORD:
            hex_var = tk.StringVar(value="0x00000000")
            dec_var = tk.StringVar(value="0")
            ttk.Label(frame, text="Hex:").grid(row=0, column=0, padx=6, pady=4, sticky="e")
            hex_entry = ttk.Entry(frame, textvariable=hex_var, width=16)
            hex_entry.grid(row=0, column=1, padx=6, pady=4)
            ttk.Label(frame, text="Dec:").grid(row=1, column=0, padx=6, pady=4, sticky="e")
            dec_entry = ttk.Entry(frame, textvariable=dec_var, width=16)
            dec_entry.grid(row=1, column=1, padx=6, pady=4)
            _upd = [False]

            def on_hex_chg(*_):
                if _upd[0]:
                    return
                try:
                    s = hex_var.get().strip()
                    v = int(s, 16)
                    _upd[0] = True; dec_var.set(str(v & 0xFFFFFFFF)); _upd[0] = False
                except ValueError:
                    pass

            def on_dec_chg(*_):
                if _upd[0]:
                    return
                try:
                    v = int(dec_var.get().strip())
                    _upd[0] = True; hex_var.set(f"0x{v & 0xFFFFFFFF:08x}"); _upd[0] = False
                except ValueError:
                    pass

            hex_var.trace_add("write", on_hex_chg)
            dec_var.trace_add("write", on_dec_chg)

            def get_data_fn():  # type: ignore[misc]
                try:
                    s = hex_var.get().strip()
                    v = int(s, 16) if s.lower().startswith("0x") else int(s, 10)
                    return struct.pack("<I", v & 0xFFFFFFFF), REG_DWORD
                except ValueError:
                    return None, None

        elif vtype in (REG_SZ, REG_EXPAND_SZ):
            text_var = tk.StringVar()
            entry = ttk.Entry(frame, textvariable=text_var, width=48)
            entry.pack(fill="x", padx=6, pady=6)

            def get_data_fn():  # type: ignore[misc]
                encoded = (text_var.get() + "\x00").encode("utf-16le")
                return encoded, vtype

        elif vtype == REG_MULTI_SZ:
            text_box = tk.Text(frame, height=5, width=48, font=("Courier New", 10))
            text_box.pack(fill="both", expand=True, padx=6, pady=6)
            ttk.Label(frame, text="(one string per line)", foreground="gray").pack(anchor="w", padx=6)

            def get_data_fn():  # type: ignore[misc]
                lines = text_box.get("1.0", "end-1c").split("\n")
                encoded = ("\x00".join(lines) + "\x00\x00").encode("utf-16le")
                return encoded, REG_MULTI_SZ

        else:  # REG_BINARY and others
            text_box = tk.Text(frame, height=4, width=48, font=("Courier New", 10))
            text_box.pack(fill="both", expand=True, padx=6, pady=6)
            ttk.Label(frame, text="(hex bytes, space-separated)", foreground="gray").pack(anchor="w", padx=6)

            def get_data_fn():  # type: ignore[misc]
                try:
                    raw = bytes.fromhex(text_box.get("1.0", "end-1c").strip().replace(" ", ""))
                    return raw, vtype
                except ValueError:
                    return None, None

        btn_frame = ttk.Frame(dlg)
        btn_frame.grid(row=3, column=0, columnspan=2, padx=10, pady=(0, 10), sticky="e")

        def ok(_e=None):
            nm = name_var.get().strip()
            if not nm:
                from tkinter import messagebox
                messagebox.showwarning("Name required", "Enter a value name.", parent=dlg)
                return
            data, dtype = get_data_fn()  # type: ignore[misc]
            if data is None:
                from tkinter import messagebox
                messagebox.showerror("Invalid data", "Could not parse the entered data.", parent=dlg)
                return
            result[0] = (nm, data, dtype)
            dlg.destroy()

        def cancel(_e=None):
            dlg.destroy()

        ttk.Button(btn_frame, text="OK", command=ok, width=10).pack(side="left", padx=6)
        ttk.Button(btn_frame, text="Cancel", command=cancel, width=10).pack(side="left", padx=6)
        name_entry.bind("<Return>", lambda _e: frame.focus())
        dlg.bind("<Escape>", cancel)
        dlg.wait_window()
        return result[0]

    def _do_delete_key(self, iid: str, key: HiveKey) -> None:
        if self.hive.format_name != "ce_cedb":
            self._show_ce_only()
            return
        from tkinter import messagebox
        if not messagebox.askyesno(
            "Delete Key",
            f"Delete '{key.name}' and all its subkeys and values?",
            parent=self.root,
        ):
            return
        parent_item = self.tree.parent(iid)
        parent_key = self.item_to_key.get(parent_item)
        if parent_key is None:
            return
        self.hive.push_undo()
        try:
            self.hive.delete_ce_key(key, parent_key)
        except Exception as exc:
            self.hive._undo_stack.pop()
            messagebox.showerror("Error", str(exc), parent=self.root)
            return
        self._remove_tree_item(iid)
        if parent_item:
            self.tree.selection_set(parent_item)
            self._show_key(parent_key)
        self._update_dirty_title()

    def _do_delete_value(self, iid: str, key: HiveKey, value: HiveValue) -> None:
        if self.hive.format_name != "ce_cedb":
            self._show_ce_only()
            return
        from tkinter import messagebox
        if not messagebox.askyesno(
            "Delete Value",
            f"Delete value '{value.name}'?",
            parent=self.root,
        ):
            return
        self.hive.push_undo()
        try:
            self.hive.delete_ce_value(key, value)
        except Exception as exc:
            self.hive._undo_stack.pop()
            messagebox.showerror("Error", str(exc), parent=self.root)
            return
        self.values.delete(iid)
        del self.item_to_value[iid]
        sel_tree = self.tree.selection()
        if sel_tree:
            k = self.item_to_key.get(sel_tree[0])
            if k:
                self._refresh_status(k)
        self._update_dirty_title()

    # ------------------------------------------------------------------
    # Helpers
    # ------------------------------------------------------------------

    def _ask_name(self, title: str, prompt: str) -> Optional[str]:
        tk = self.tk
        ttk = self.ttk
        result: list = [None]
        dlg = tk.Toplevel(self.root)
        dlg.title(title)
        dlg.resizable(False, False)
        dlg.transient(self.root)
        dlg.grab_set()
        ttk.Label(dlg, text=prompt).grid(row=0, column=0, padx=10, pady=(12, 4), sticky="w")
        var = tk.StringVar()
        entry = ttk.Entry(dlg, textvariable=var, width=40)
        entry.grid(row=1, column=0, columnspan=2, padx=10, pady=(0, 8), sticky="ew")
        entry.focus()

        def ok(_e=None):
            v = var.get().strip()
            if v:
                result[0] = v
                dlg.destroy()

        def cancel(_e=None):
            dlg.destroy()

        btn_frame = ttk.Frame(dlg)
        btn_frame.grid(row=2, column=0, columnspan=2, pady=(0, 10))
        ttk.Button(btn_frame, text="OK", command=ok, width=10).pack(side="left", padx=6)
        ttk.Button(btn_frame, text="Cancel", command=cancel, width=10).pack(side="left", padx=6)
        entry.bind("<Return>", ok)
        dlg.bind("<Escape>", cancel)
        dlg.wait_window()
        return result[0]

    def _show_ce_only(self) -> None:
        from tkinter import messagebox
        messagebox.showinfo(
            "CE only",
            "Add/delete operations are only supported for CE CEDB hives.",
            parent=self.root,
        )

    def _remove_tree_item(self, iid: str) -> None:
        for child in self.tree.get_children(iid):
            self._remove_tree_item(child)
        if iid in self.item_to_key:
            del self.item_to_key[iid]
        self.tree.delete(iid)

    def _refresh_status(self, key: HiveKey) -> None:
        path = self._selected_path()
        timestamp = key.last_write.isoformat() if key.last_write else "unknown time"
        self.status_var.set(
            f"{path}    {len(key.children)} subkeys, {len(key.values)} values, "
            f"cell 0x{key.offset:x}, last write {timestamp}"
        )

    def _on_close(self) -> None:
        if self.hive.dirty:
            from tkinter import messagebox
            choice = messagebox.askyesnocancel(
                "Unsaved changes", "Save changes before closing?", parent=self.root
            )
            if choice is None:
                return
            if choice:
                self._save()
        self.root.destroy()


def summarize_hive(hive: RegistryHive) -> str:
    keys = 0
    values = 0
    for _path, key in hive.iter_keys():
        keys += 1
        values += len(key.values)
    crc = binascii.crc32(hive.data) & 0xFFFFFFFF
    return (
        f"{hive.path}\n"
        f"  format: {hive.format_name}\n"
        f"  root: {hive.root.name}\n"
        f"  source name: {hive.filename or '(none)'}\n"
        f"  hive bins: {hive.hbins_size} bytes\n"
        f"  keys: {keys}\n"
        f"  values: {values}\n"
        f"  crc32: 0x{crc:08x}"
    )


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Parse and view Windows CE/NT registry .hv hives.")
    parser.add_argument("hive", help="Path to the .hv hive file.")
    parser.add_argument("--root", default="HKLM", help="Root alias for export/display, e.g. HKLM, HKCU, HKCR.")
    parser.add_argument("--export-json", help="Write this repo's regs.json-compatible flattened JSON.")
    parser.add_argument("--summary", action="store_true", help="Print hive summary and exit unless GUI is requested.")
    parser.add_argument("--no-gui", action="store_true", help="Do not open the Tk viewer.")
    return parser.parse_args(argv)


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = parse_args(argv)
    try:
        hive = RegistryHive(args.hive, args.root)
    except Exception as exc:
        print(f"hv_viewer: {exc}", file=sys.stderr)
        return 2

    if args.summary or args.no_gui or args.export_json:
        print(summarize_hive(hive))

    if args.export_json:
        with open(args.export_json, "w", encoding="utf-8") as f:
            json.dump(hive.to_regs_json(), f, indent=2, ensure_ascii=False)
            f.write("\n")
        print(f"exported {args.export_json}")

    if args.no_gui or args.summary and not args.export_json:
        return 0

    try:
        HiveViewer(hive).run()
    except ImportError as exc:
        print(f"hv_viewer: Tkinter is unavailable: {exc}", file=sys.stderr)
        return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
