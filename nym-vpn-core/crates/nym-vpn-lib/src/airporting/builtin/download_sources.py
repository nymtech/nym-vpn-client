#!/usr/bin/env python3
"""
Read IP address info from ipinfo.io, filter by country and write the
IP address ranges into a gzip-compressed text file for processing by Nym VPN.

To add more countries, add their ISO country codes to COUNTRY_CODES.
Each country produces its own airporting_XX.txt.gz and airporting_XX.txt.meta.
"""

import gzip
import hashlib
import ipaddress
import json
import sys
import urllib.request
from contextlib import contextmanager
from datetime import datetime, timezone

IPINFO_TOKEN = "0cb655a56b9b47"
IPINFO_URL = f"https://ipinfo.io/data/ipinfo_lite.json.gz?token={IPINFO_TOKEN}"
COUNTRY_CODES = {"CN"}


@contextmanager
def open_input(source):
    """Open a local file or URL, decompressing gzip automatically."""
    if source.startswith("http://") or source.startswith("https://"):
        path = source.split("?")[0]  # strip query string for extension check
        with urllib.request.urlopen(source) as resp:
            if path.endswith(".gz"):
                with gzip.GzipFile(fileobj=resp) as gz:
                    yield (line.decode() for line in gz)
            else:
                yield (line.decode() for line in resp)
    elif source.endswith(".gz"):
        with gzip.open(source, "rt") as f:
            yield f
    else:
        with open(source) as f:
            yield f


def write_country(code, networks):
    """Collapse networks for a single country and write output files."""
    ipv4 = [n for n in networks if n.version == 4]
    ipv6 = [n for n in networks if n.version == 6]
    collapsed = list(ipaddress.collapse_addresses(ipv4)) + list(ipaddress.collapse_addresses(ipv6))
    print(f"  Collapsed to {len(collapsed)} networks ({len(ipv4)} IPv4, {len(ipv6)} IPv6)", file=sys.stderr)

    text_bytes = "".join(f"{n}\n" for n in collapsed).encode("utf-8")

    output_file = f"airporting_{code.lower()}.txt.gz"
    meta_file = f"airporting_{code.lower()}.txt.meta"

    with gzip.open(output_file, "wb", compresslevel=9) as f:
        f.write(text_bytes)
    print(f"  Written to {output_file}", file=sys.stderr)

    meta = {
        "updated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "line_count": len(collapsed),
        "length": len(text_bytes),
        "sha256": hashlib.sha256(text_bytes).hexdigest(),
    }
    with open(meta_file, "w") as f:
        json.dump(meta, f, indent=2)
        f.write("\n")
    print(f"  Written to {meta_file}", file=sys.stderr)


# Collect networks per country in a single pass over the source data
networks_by_country = {code: [] for code in COUNTRY_CODES}
with open_input(IPINFO_URL) as lines:
    for lineno, line in enumerate(lines, 1):
        line = line.strip()
        if not line:
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError as e:
            print(f"Warning: skipping malformed line {lineno}: {e}", file=sys.stderr)
            continue
        code = record.get("country_code")
        if code in networks_by_country:
            try:
                networks_by_country[code].append(ipaddress.ip_network(record["network"], strict=False))
            except ValueError as e:
                print(f"Warning: invalid network on line {lineno}: {e}", file=sys.stderr)

for code in sorted(COUNTRY_CODES):
    networks = networks_by_country[code]
    print(f"{code}: found {len(networks)} networks before collapsing", file=sys.stderr)
    write_country(code, networks)
