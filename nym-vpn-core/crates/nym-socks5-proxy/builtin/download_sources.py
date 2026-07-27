#!/usr/bin/env python3
"""Download socks5 proxy routing sources from the CDN (already gzip-compressed) and
save them along with their ETag.  Run this script to refresh the builtin files
embedded in the binary.

The .gz files are embedded via include_bytes! and the .etag files are embedded via
include_str! so that the first periodic update check can send If-None-Match and skip
the download if nothing has changed.
"""

from __future__ import annotations

import urllib.error
import urllib.request
from pathlib import Path


BASE_URL = "https://geo-exclusion.sos-ch-gva-2.exoscale-cdn.com"

# Country codes whose routing files should be downloaded.
# For each code XX the script expects XX-ip.json.gz and XX-domain.txt.gz on the CDN.
COUNTRY_CODES: list[str] = ["CN", "RU"]


def _sources_for(country_code: str) -> list[str]:
    cc = country_code.upper()
    return [f"{cc}-ip.json.gz", f"{cc}-domain.txt.gz"]


SOURCES: list[str] = [f for cc in COUNTRY_CODES for f in _sources_for(cc)]

USER_AGENT = "nym-socks5-proxy/1.0 (+download_sources.py)"
TIMEOUT_SECS = 60


def _read_etag(etag_path: Path) -> str | None:
    """Return the stored ETag string, or None if the file does not exist."""
    try:
        return etag_path.read_text(encoding="utf-8").strip() or None
    except FileNotFoundError:
        return None


def _head_etag(url: str) -> str | None:
    """Return the ETag from a HEAD request, or None if unavailable."""
    req = urllib.request.Request(url, method="HEAD", headers={"User-Agent": USER_AGENT})
    try:
        resp = urllib.request.urlopen(req, timeout=TIMEOUT_SECS)
        return resp.headers.get("ETag") or None
    except urllib.error.URLError:
        return None


def _download(url: str, dest: Path) -> str:
    """Download *url* to *dest* and return the ETag from the response.

    Raises ``SystemExit`` if the response does not include an ETag.
    """
    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": USER_AGENT,
            "Accept": "application/gzip, */*;q=0.9",
        },
    )

    try:
        resp = urllib.request.urlopen(req, timeout=TIMEOUT_SECS)
    except urllib.error.URLError as exc:
        raise SystemExit(f"Failed to download {url}: {exc}") from exc

    etag = resp.headers.get("ETag")
    if not etag:
        raise SystemExit(f"No ETag in response from {url}")

    tmp = dest.with_suffix(dest.suffix + ".tmp")
    tmp.parent.mkdir(parents=True, exist_ok=True)

    with tmp.open("wb") as out:
        while chunk := resp.read(1024 * 1024):
            out.write(chunk)

    tmp.replace(dest)
    return etag


def main() -> int:
    wrote = 0
    for filename in SOURCES:
        url = f"{BASE_URL}/{filename}"
        gz_path = Path(filename)
        etag_path = gz_path.with_suffix(".etag")

        # Step 1: HEAD request to read the server's current ETag without downloading.
        current_etag = _read_etag(etag_path)
        if current_etag:
            server_etag = _head_etag(url)

            # Step 2 & 3: Compare — match → skip the download entirely.
            if server_etag == current_etag:
                print(f"Skipped {gz_path} (not modified, etag={current_etag!r})")
                continue

        # Step 4: Different (or no stored etag) → full GET, write file, store new ETag.
        new_etag = _download(url, gz_path)
        etag_path.write_text(new_etag, encoding="utf-8")
        print(f"Wrote {gz_path}")
        print(f"Wrote {etag_path} (etag={new_etag!r})")
        wrote += 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
