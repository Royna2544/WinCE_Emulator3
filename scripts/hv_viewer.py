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
    offset: int
    parent_offset: int
    last_write: Optional[datetime]
    values: List[HiveValue] = field(default_factory=list)
    children: List["HiveKey"] = field(default_factory=list)


class RegistryHive:
    def __init__(self, path: str, root_name: str = "HKLM") -> None:
        self.path = path
        self.root_name = normalize_root(root_name)
        with open(path, "rb") as f:
            self.data = f.read()
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
        if raw_data_len & 0x80000000:
            data = struct.pack("<I", data_offset)[:data_len]
        elif data_len == 0 or data_offset == 0xFFFFFFFF:
            data = b""
        else:
            data_payload = self._cell_payload_offset(data_offset)
            data = self._bytes(data_payload, data_len)
        return HiveValue(name=name, value_type=value_type, data=data)

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
        for record in records:
            if record["kind"] != "key":
                continue
            oid = int(record["oid"])
            key = keys[oid]
            for child_oid in reversed(self._walk_ce_record_chain(int(record["last_child_oid"]), key_records)):
                child = keys.get(child_oid)
                if child is None or child is key:
                    continue
                child.parent_offset = oid
                key.children.append(child)
                assigned_children.add(child_oid)

        for record in records:
            if record["kind"] == "key":
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
            "data": self._bytes(value_offset, data_len),
        }

    def _sort_tree(self, key: HiveKey) -> None:
        key.children.sort(key=lambda child: child.name.lower())
        key.values.sort(key=lambda value: value.name.lower())
        for child in key.children:
            self._sort_tree(child)

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

        self.root = tk.Tk()
        self.root.title(f"Registry Hive Viewer - {os.path.basename(hive.path)}")
        self.root.geometry("1100x700")
        self.root.minsize(720, 420)

        self._build_menu()
        self._build_body()
        self._populate_tree()

    def run(self) -> None:
        self.root.mainloop()

    def _build_menu(self) -> None:
        menu = self.tk.Menu(self.root)
        file_menu = self.tk.Menu(menu, tearoff=False)
        file_menu.add_command(label="Open Hive...", command=self._open_hive)
        file_menu.add_command(label="Export JSON...", command=self._export_json)
        file_menu.add_separator()
        file_menu.add_command(label="Exit", command=self.root.destroy)
        menu.add_cascade(label="File", menu=file_menu)
        self.root.config(menu=menu)

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

        self.status_var = self.tk.StringVar(value="")
        status = self.ttk.Label(outer, textvariable=self.status_var, anchor="w")
        status.pack(fill="x", padx=6, pady=(4, 6))

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
        for value in key.values:
            self.values.insert("", "end", values=(value.name, value.type_name, value.display_data()))
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
