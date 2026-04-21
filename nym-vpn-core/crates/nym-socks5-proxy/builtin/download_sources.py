#!/usr/bin/env python3

from __future__ import annotations

import gzip
import ipaddress
import json
import math
import urllib.error
import urllib.request
from pathlib import Path

APNIC_DELEGATED_URL = "https://ftp.apnic.net/stats/apnic/delegated-apnic-latest"
DNSMASQ_CHINA_URL = (
    "https://raw.githubusercontent.com/felixonmars/dnsmasq-china-list"
    "/master/accelerated-domains.china.conf"
)

USER_AGENT = "nym-socks5-proxy/1.0 (+download_sources.py)"
TIMEOUT_SECS = 60


def _fetch(url: str) -> bytes:
    req = urllib.request.Request(
        url,
        headers={"User-Agent": USER_AGENT, "Accept": "*/*"},
    )
    try:
        with urllib.request.urlopen(req, timeout=TIMEOUT_SECS) as resp:
            return resp.read()
    except urllib.error.URLError as e:
        raise SystemExit(f"Failed to download {url}: {e}") from e


def _write_gz(path: Path, data: bytes) -> None:
    tmp = path.with_suffix(path.suffix + ".tmp")
    with gzip.open(tmp, "wb", compresslevel=9) as f:
        f.write(data)
    tmp.replace(path)


def download_cn_ip() -> None:
    print(f"Downloading CN IP ranges from:\n  {APNIC_DELEGATED_URL}")
    raw = _fetch(APNIC_DELEGATED_URL).decode("ascii", errors="ignore")

    ipv4: list[str] = []
    ipv6: list[str] = []

    for line in raw.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue

        parts = line.split("|")
        if len(parts) < 6:
            continue

        _registry, country, rtype, start, value = parts[:5]
        if country.upper() != "CN":
            continue

        if rtype == "ipv4":
            try:
                count = int(value)
                log = math.log2(count)
                if log != int(log):
                    continue  # skip non-power-of-2 counts
                prefix_len = 32 - int(log)
                net = ipaddress.IPv4Network(f"{start}/{prefix_len}", strict=False)
                ipv4.append(str(net))
            except (ValueError, ArithmeticError):
                continue

        elif rtype == "ipv6":
            try:
                net = ipaddress.IPv6Network(f"{start}/{value}", strict=False)
                ipv6.append(str(net))
            except ValueError:
                continue

    ipv4.sort(key=lambda n: ipaddress.IPv4Network(n))
    ipv6.sort(key=lambda n: ipaddress.IPv6Network(n))

    out = json.dumps({"ipv4": ipv4, "ipv6": ipv6}, ensure_ascii=True).encode()
    _write_gz(Path("CN-ip.json.gz"), out)
    print(f"  Written CN-ip.json.gz ({len(ipv4)} IPv4, {len(ipv6)} IPv6 ranges)")


def download_cn_domains() -> None:
    print(f"Downloading CN domain list from:\n  {DNSMASQ_CHINA_URL}")
    raw = _fetch(DNSMASQ_CHINA_URL).decode("utf-8", errors="ignore")

    domains: list[str] = []
    for line in raw.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        # Format: server=/domain.tld/114.114.114.114
        if line.startswith("server=/"):
            parts = line.split("/")
            if len(parts) >= 2:
                domain = parts[1].strip()
                if domain:
                    domains.append(domain)

    domains.sort()
    out = "\n".join(domains).encode("utf-8")
    _write_gz(Path("CN-domain.txt.gz"), out)
    print(f"  Written CN-domain.txt.gz ({len(domains)} domains)")


def main() -> int:
    download_cn_ip()
    download_cn_domains()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
