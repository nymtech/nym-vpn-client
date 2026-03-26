#!/usr/bin/env python3
"""
Read IP address info from ipinfo.io, filter by country and write the
IP address ranges into a gzip-compressed text file for processing by Nym VPN.
"""

import gzip
import hashlib
import ipaddress
import json
import sys  # used for stderr
import urllib.request
from contextlib import contextmanager
from datetime import datetime, timezone

IPINFO_TOKEN = "0cb655a56b9b47"
IPINFO_URL = f"https://ipinfo.io/data/ipinfo_lite.json.gz?token={IPINFO_TOKEN}"
COUNTRY_CODES = {"CN"}
OUTPUT_FILE = "airporting.txt.gz"
META_FILE = "airporting.txt.meta"

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


networks = []
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
        if record.get("country_code") in COUNTRY_CODES:
            try:
                networks.append(ipaddress.ip_network(record["network"], strict=False))
            except ValueError as e:
                print(f"Warning: invalid network on line {lineno}: {e}", file=sys.stderr)

codes = ", ".join(sorted(COUNTRY_CODES))
print(f"Found {len(networks)} networks for {codes} before collapsing.", file=sys.stderr)

ipv4 = [n for n in networks if n.version == 4]
ipv6 = [n for n in networks if n.version == 6]
collapsed = list(ipaddress.collapse_addresses(ipv4)) + list(ipaddress.collapse_addresses(ipv6))

print(f"Collapsed to {len(collapsed)} networks ({sum(1 for n in collapsed if n.version==4)} IPv4, {sum(1 for n in collapsed if n.version==6)} IPv6).", file=sys.stderr)

plain_text = "".join(f"{n}\n" for n in collapsed)
text_bytes = plain_text.encode("utf-8")
length = len(text_bytes)
sha256 = hashlib.sha256(text_bytes).hexdigest()

with gzip.open(OUTPUT_FILE, "wb", compresslevel=9) as f:
    f.write(text_bytes)
print(f"Written to {OUTPUT_FILE}", file=sys.stderr)

meta = {
    "updated_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "line_count": len(collapsed),
    "length": length,
    "sha256": sha256,
}
with open(META_FILE, "w") as f:
    json.dump(meta, f, indent=2)
    f.write("\n")
print(f"Written to {META_FILE}", file=sys.stderr)
