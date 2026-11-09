#!/usr/bin/env python3
"""Compare Android native library builds and explain .gcc_except_table differences.

Usage: compare.py BASELINE_DIR OTHER_DIR [OTHER_DIR...]

Each directory holds one subdirectory per ABI, e.g. arm64-v8a, with:
  stripped.so    library as packaged (required)
  unstripped.so  library before stripping (optional)
  link.map       lld map of the final link (optional)

Compares every ABI present in either directory. Prints a Markdown report to
stdout. Exits with status 1 if any library is missing or differs from the
baseline.
"""

import bisect
import hashlib
import re
import struct
import sys
from pathlib import Path

from elftools.elf.elffile import ELFFile

EXCEPT_TABLE = ".gcc_except_table"
MAP_LINE = re.compile(r"^\s*([0-9a-f]+)\s+([0-9a-f]+)\s+([0-9a-f]+)\s+(\d+)( +)(\S.*)$")
HEX_RUN = re.compile(r"[0-9a-f]{8,}")
LLVM_SUFFIX = re.compile(r"\.llvm\.\d+")
MAX_LISTED = 40


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Lib:
    def __init__(self, abi_dir):
        with (abi_dir / "stripped.so").open("rb") as f:
            elf = ELFFile(f)
            self.elfclass = elf.elfclass
            self.little = elf.little_endian
            self.sections = {}
            for s in elf.iter_sections():
                if s.name:
                    content = b"" if s["sh_type"] == "SHT_NOBITS" else s.data()
                    self.sections[s.name] = (s["sh_addr"], content)
        self.symbols = {}
        unstripped = abi_dir / "unstripped.so"
        if unstripped.exists():
            with unstripped.open("rb") as f:
                symtab = ELFFile(f).get_section_by_name(".symtab")
                if symtab is not None:
                    for sym in symtab.iter_symbols():
                        if sym.name and sym["st_info"]["type"] == "STT_FUNC":
                            self.symbols[sym.name] = sym["st_value"]
        map_path = abi_dir / "link.map"
        self.map = parse_map(map_path) if map_path.exists() else None
        self.kept_text = {section_of(t) for _, _, t in self.map.get(".text", [])} if self.map else set()
        self.lsdas = parse_eh_frame_lsdas(self)
        self.lsda_addrs = sorted(self.lsdas)

    def section(self, name):
        return self.sections.get(name)


def parse_map(path):
    """Return {output_section: [(vma, size, input_description)]} from an lld map."""
    result = {}
    current = None
    for line in path.read_text(errors="replace").splitlines():
        m = MAP_LINE.match(line)
        if not m:
            continue
        vma, size, indent, text = int(m.group(1), 16), int(m.group(3), 16), len(m.group(5)), m.group(6)
        level = (indent - 1) // 8
        if level == 0:
            current = text.split()[0]
            result.setdefault(current, [])
        elif level == 1 and current is not None:
            result[current].append((vma, size, text))
    return result


def normalize_input(text):
    """Strip build-specific paths and hashes from an lld map input description."""
    m = re.match(r"^(.*):\((.*)\)$", text)
    if not m:
        return HEX_RUN.sub("*", text)
    obj, section = m.groups()
    obj = obj.rsplit("/", 1)[-1]
    return f"{HEX_RUN.sub('*', obj)}:({LLVM_SUFFIX.sub('.llvm.*', section)})"


def section_of(text):
    m = re.match(r"^.*:\((.*)\)$", text)
    return m.group(1) if m else text


def read_uleb(data, pos):
    result = shift = 0
    while True:
        byte = data[pos]
        pos += 1
        result |= (byte & 0x7F) << shift
        shift += 7
        if not byte & 0x80:
            return result, pos


def read_sleb(data, pos):
    result = shift = 0
    while True:
        byte = data[pos]
        pos += 1
        result |= (byte & 0x7F) << shift
        shift += 7
        if not byte & 0x80:
            if byte & 0x40:
                result -= 1 << shift
            return result, pos


def read_encoded(lib, data, pos, base_vma, enc):
    """Decode a DW_EH_PE-encoded pointer. Returns (value, raw, new_pos)."""
    e = "<" if lib.little else ">"
    fmt = enc & 0x0F
    field_vma = base_vma + pos
    if fmt == 0x00:
        size = lib.elfclass // 8
        raw = struct.unpack_from(e + ("Q" if size == 8 else "I"), data, pos)[0]
        pos += size
    elif fmt == 0x01:
        raw, pos = read_uleb(data, pos)
    elif fmt == 0x09:
        raw, pos = read_sleb(data, pos)
    else:
        code, size = {0x02: ("H", 2), 0x03: ("I", 4), 0x04: ("Q", 8),
                      0x0A: ("h", 2), 0x0B: ("i", 4), 0x0C: ("q", 8)}[fmt]
        raw = struct.unpack_from(e + code, data, pos)[0]
        pos += size
    value = raw + field_vma if enc & 0x70 == 0x10 else raw
    return value, raw, pos


def parse_eh_frame_lsdas(lib):
    """Return {lsda_vma: pc_begin} for every FDE in .eh_frame that has an LSDA."""
    section = lib.section(".eh_frame")
    if section is None:
        return {}
    base, data = section
    e = "<" if lib.little else ">"
    cies = {}
    lsdas = {}
    pos = 0
    while pos + 4 <= len(data):
        start = pos
        length = struct.unpack_from(e + "I", data, pos)[0]
        pos += 4
        if length == 0:
            continue
        if length == 0xFFFFFFFF:
            length = struct.unpack_from(e + "Q", data, pos)[0]
            pos += 8
        end = pos + length
        id_pos = pos
        cie_id = struct.unpack_from(e + "I", data, pos)[0]
        pos += 4
        try:
            if cie_id == 0:
                version = data[pos]
                pos += 1
                aug_end = data.index(b"\0", pos)
                aug = data[pos:aug_end].decode()
                pos = aug_end + 1
                _, pos = read_uleb(data, pos)
                _, pos = read_sleb(data, pos)
                if version == 1:
                    pos += 1
                else:
                    _, pos = read_uleb(data, pos)
                cie = {"aug": aug, "fde_enc": 0x00, "lsda_enc": None}
                if aug.startswith("z"):
                    _, pos = read_uleb(data, pos)
                    for ch in aug[1:]:
                        if ch == "L":
                            cie["lsda_enc"] = data[pos]
                            pos += 1
                        elif ch == "P":
                            p_enc = data[pos]
                            pos += 1
                            _, _, pos = read_encoded(lib, data, pos, base, p_enc)
                        elif ch == "R":
                            cie["fde_enc"] = data[pos]
                            pos += 1
                        elif ch in "SB":
                            continue
                        else:
                            break
                cies[start] = cie
            else:
                cie = cies.get(id_pos - cie_id)
                if cie is not None:
                    pc_begin, _, pos = read_encoded(lib, data, pos, base, cie["fde_enc"])
                    _, _, pos = read_encoded(lib, data, pos, base, cie["fde_enc"] & 0x0F)
                    if cie["aug"].startswith("z"):
                        _, pos = read_uleb(data, pos)
                        if cie["lsda_enc"] not in (None, 0xFF):
                            lsda, raw, pos = read_encoded(lib, data, pos, base, cie["lsda_enc"])
                            if raw != 0:
                                lsdas[lsda] = pc_begin
        except (IndexError, KeyError, struct.error, UnicodeDecodeError):
            pass
        pos = end
    return lsdas


def divergence(a, b):
    """Return (first differing offset, last differing offset in a, in b) or None."""
    if a == b:
        return None
    prefix = 0
    limit = min(len(a), len(b))
    while prefix < limit and a[prefix] == b[prefix]:
        prefix += 1
    suffix = 0
    while suffix < limit - prefix and a[-1 - suffix] == b[-1 - suffix]:
        suffix += 1
    return prefix, len(a) - suffix, len(b) - suffix


def covering(entries, start, end):
    return [entry for entry in entries if entry[0] < end and start < entry[0] + entry[1]]


def referenced(entry, lib):
    vma, size, _ = entry
    lo = bisect.bisect_left(lib.lsda_addrs, vma)
    hi = bisect.bisect_left(lib.lsda_addrs, vma + size)
    return [lib.lsdas[a] for a in lib.lsda_addrs[lo:hi]]


def describe_entries(lib, entries, out):
    for vma, size, text in entries[:MAX_LISTED]:
        refs = referenced((vma, size, text), lib)
        name = section_of(text)
        func = name[len(EXCEPT_TABLE) + 1:] if name.startswith(EXCEPT_TABLE + ".") else None
        notes = [f"referenced by {len(refs)} FDE(s)" if refs else "**not referenced by any FDE**"]
        if func is not None and lib.map is not None:
            text_kept = f".text.{func}" in lib.kept_text
            notes.append(f"`.text.{func[:16]}…` {'kept' if text_kept else '**not in map**'}")
        if func is not None and lib.symbols:
            addr = lib.symbols.get(func)
            if addr is None:
                notes.append("function symbol **absent**")
            else:
                aliases = [s for s, a in lib.symbols.items() if a == addr and s != func]
                notes.append(f"function at {addr:#x}" + (f", {len(aliases)} alias(es) at same address" if aliases else ""))
        out.append(f"    - `{vma:#x}+{size:#x}` `{normalize_input(text)}` — {', '.join(notes)}")
    if len(entries) > MAX_LISTED:
        out.append(f"    - … {len(entries) - MAX_LISTED} more")


def compare_maps(a, b, out):
    ea = a.map.get(EXCEPT_TABLE, [])
    eb = b.map.get(EXCEPT_TABLE, [])
    na = [normalize_input(t) for _, _, t in ea]
    nb = [normalize_input(t) for _, _, t in eb]
    out.append(f"  - map entries in `{EXCEPT_TABLE}`: {len(na)} vs {len(nb)}")
    first = next((i for i, (x, y) in enumerate(zip(na, nb)) if x != y), None)
    if first is None and len(na) != len(nb):
        first = min(len(na), len(nb))
    if first is not None:
        out.append(f"  - entry order diverges at index {first}: `{na[first] if first < len(na) else '—'}` vs `{nb[first] if first < len(nb) else '—'}`")
    sizes_a, sizes_b = {}, {}
    for (_, size, _), name in zip(ea, na):
        sizes_a.setdefault(name, []).append(size)
    for (_, size, _), name in zip(eb, nb):
        sizes_b.setdefault(name, []).append(size)
    only_a = sorted(set(sizes_a) - set(sizes_b))
    only_b = sorted(set(sizes_b) - set(sizes_a))
    resized = sorted(n for n in set(sizes_a) & set(sizes_b) if sizes_a[n] != sizes_b[n])
    for label, names in (("only in baseline", only_a), ("only in other", only_b)):
        if names:
            out.append(f"  - {len(names)} input section(s) {label}:")
            out.extend(f"    - `{n}`" for n in names[:MAX_LISTED])
    if resized:
        out.append(f"  - {len(resized)} input section(s) changed size:")
        out.extend(f"    - `{n}`: {[hex(s) for s in sizes_a[n]]} → {[hex(s) for s in sizes_b[n]]}" for n in resized[:MAX_LISTED])
    for label, lib, entries in (("baseline", a, ea), ("other", b, eb)):
        if not lib.lsdas:
            continue
        orphans = [entry for entry in entries if not referenced(entry, lib)]
        dead = [e for e in orphans if f".text.{section_of(e[2])[len(EXCEPT_TABLE) + 1:]}" not in lib.kept_text]
        out.append(
            f"  - {label}: {len(orphans)} of {len(entries)} `{EXCEPT_TABLE}` input sections are not referenced by any FDE, "
            f"{len(dead)} of them belong to functions whose `.text` was garbage-collected"
        )


def compare_abi(base_dir, other_dir, abi, out):
    """Report how the ABI's libraries compare. Returns True if they're identical."""
    if not (base_dir / abi / "stripped.so").exists() or not (other_dir / abi / "stripped.so").exists():
        out.append(f"- **{abi}**: missing library ❌")
        return False
    if sha256(base_dir / abi / "stripped.so") == sha256(other_dir / abi / "stripped.so"):
        out.append(f"- **{abi}**: identical ✅")
        return True
    out.append(f"- **{abi}**: differs ❌")
    explain(Lib(base_dir / abi), Lib(other_dir / abi), out)
    return False


def explain(a, b, out):
    names = [n for n in a.sections if n in b.sections] + [n for n in b.sections if n not in a.sections]
    for name in names:
        sa, sb = a.section(name), b.section(name)
        if sa is None or sb is None:
            out.append(f"  - `{name}` only in {'other' if sa is None else 'baseline'}")
        elif sa[1] != sb[1] or sa[0] != sb[0]:
            out.append(f"  - `{name}` differs: addr {sa[0]:#x} vs {sb[0]:#x}, size {len(sa[1]):#x} vs {len(sb[1]):#x}")
    sa, sb = a.section(EXCEPT_TABLE), b.section(EXCEPT_TABLE)
    if sa is None or sb is None:
        return
    div = divergence(sa[1], sb[1])
    if div is None:
        out.append(f"  - `{EXCEPT_TABLE}` content identical")
        return
    first, end_a, end_b = div
    out.append(
        f"  - `{EXCEPT_TABLE}` diverges from offset {first:#x}: "
        f"baseline {sa[0] + first:#x}..{sa[0] + end_a:#x}, other {sb[0] + first:#x}..{sb[0] + end_b:#x} "
        f"(FDEs with LSDA: {len(a.lsdas)} vs {len(b.lsdas)})"
    )
    for label, lib, (addr, _), end in (("baseline", a, sa, end_a), ("other", b, sb, end_b)):
        if lib.map is None:
            continue
        entries = covering(lib.map.get(EXCEPT_TABLE, []), addr + first, addr + max(end, first + 1))
        out.append(f"  - {label} input sections covering the first {min(len(entries), MAX_LISTED)} of {len(entries)} in the divergent range:")
        describe_entries(lib, entries, out)
    if a.map is not None and b.map is not None:
        compare_maps(a, b, out)


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    base = Path(sys.argv[1])
    out = [f"# Android native library comparison against `{base.name}`", ""]
    identical = True
    for other in map(Path, sys.argv[2:]):
        out.append(f"## `{other.name}`")
        abis = sorted({d.name for root in (base, other) for d in root.iterdir() if d.is_dir()})
        if not abis:
            out.append("- no libraries found ❌")
            identical = False
        for abi in abis:
            identical &= compare_abi(base, other, abi, out)
        out.append("")
    print("\n".join(out))
    sys.exit(0 if identical else 1)


if __name__ == "__main__":
    main()
