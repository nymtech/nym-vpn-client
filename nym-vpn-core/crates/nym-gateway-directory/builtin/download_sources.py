#!/usr/bin/env python3
"""Fetch gateway lists from the nym-vpn-api directory endpoints and save a single
deduplicated, gzip-compressed snapshot. Run this script to refresh the builtin
gateway list embedded in the binary.
"""

from __future__ import annotations

import gzip
import json
import urllib.error
import urllib.request
from pathlib import Path

BASE_URL = "https://nymvpn.com/api"

# type tag -> path (and optional query string) on nym-vpn-api
ENDPOINTS: dict[str, str] = {
    "entry": "public/v1/directory/gateways/entry",
    "exit": "public/v1/directory/gateways/exit",
    "wg": "public/v1/directory/gateways?show_vpn_only=true",
}

OUTPUT_FILE = Path("gateways.json.gz")
USER_AGENT = "nym-gateway-directory-builtin/1.0 (+download_sources.py)"
TIMEOUT_SECS = 60


def _fetch(path: str) -> list[dict]:
    url = f"{BASE_URL}/{path}"
    req = urllib.request.Request(
        url,
        headers={"User-Agent": USER_AGENT, "Accept": "application/json"},
    )
    try:
        with urllib.request.urlopen(req, timeout=TIMEOUT_SECS) as resp:
            return json.loads(resp.read())
    except urllib.error.URLError as exc:
        raise SystemExit(f"Failed to fetch {url}: {exc}") from exc


def _existing_entry_count() -> int | None:
    if not OUTPUT_FILE.exists():
        return None
    try:
        with gzip.open(OUTPUT_FILE, "rb") as f:
            return len(json.loads(f.read()))
    except (OSError, json.JSONDecodeError):
        return None


def main() -> None:
    fetched = {tag: _fetch(path) for tag, path in ENDPOINTS.items()}
    ids_by_type = {tag: {gw["identity_key"] for gw in gws} for tag, gws in fetched.items()}

    # Union of gateway objects across all endpoints, keyed by identity so a gateway that
    # shows up in more than one list is stored exactly once.
    gateways_by_id: dict[str, dict] = {}
    for gws in fetched.values():
        for gw in gws:
            gateways_by_id.setdefault(gw["identity_key"], gw)

    old_count = _existing_entry_count()

    combined = [
        {
            "types": sorted(tag for tag, ids in ids_by_type.items() if identity in ids),
            "gateway": gateway,
        }
        for identity, gateway in sorted(gateways_by_id.items())
    ]

    # Deterministic bytes regardless of API response ordering, so the committed snapshot only
    # changes when the underlying gateway data actually changes.
    raw = json.dumps(combined, sort_keys=True, separators=(",", ":")).encode("utf-8")
    tmp = OUTPUT_FILE.with_suffix(OUTPUT_FILE.suffix + ".tmp")
    with gzip.GzipFile(filename=tmp, mode="wb", mtime=0) as out:
        out.write(raw)

    counts = ", ".join(f"{tag}={len(ids)}" for tag, ids in ids_by_type.items())
    if OUTPUT_FILE.exists() and OUTPUT_FILE.read_bytes() == tmp.read_bytes():
        tmp.unlink()
        print(f"Unchanged {OUTPUT_FILE} ({len(combined)} gateways; {counts})")
        return

    tmp.replace(OUTPUT_FILE)
    if old_count is None:
        print(f"Wrote {OUTPUT_FILE} ({len(combined)} gateways; {counts})")
    else:
        print(f"Updated {OUTPUT_FILE} ({old_count} -> {len(combined)} gateways; {counts})")


if __name__ == "__main__":
    main()
