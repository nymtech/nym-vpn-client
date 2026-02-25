#!/usr/bin/env python3
"""Download blocklist sources and store them compressed.
"""

from __future__ import annotations

import argparse
import datetime as _dt
import email.utils
import gzip
import hashlib
import json
import urllib.error
import urllib.request
from dataclasses import asdict, dataclass
from pathlib import Path


SOURCES: list[tuple[str, str]] = [
    (
        "easylist_adservers.txt",
        "https://raw.githubusercontent.com/easylist/easylist/refs/heads/master/easylist/easylist_adservers.txt",
    ),
    (
        "light.txt",
        "https://cdn.jsdelivr.net/gh/hagezi/dns-blocklists@latest/hosts/light.txt",
    ),
]

USER_AGENT = "ad-blocker-lists/1.0 (+download_sources.py)"
TIMEOUT_SECS = 60


def _normalize_etag(value: str | None) -> str:
    """Return a canonical HTTP entity-tag string.

    Strips weak-vs-strong semantics (i.e. removes any W/ prefix) and
    normalizes whitespace and quoting so stored ETags are comparable.

    The returned value is the bare tag (no surrounding quotes).
    """

    if not value:
        return ""

    s = value.strip()
    if not s:
        return ""

    # Drop the weak validator prefix so all stored tags share the same format.
    if len(s) >= 2 and s[:2].lower() == "w/":
        s = s[2:].lstrip()

    # RFC-style ETags are quoted strings; tolerate unquoted values.
    if len(s) >= 2 and ((s[0] == '"' and s[-1] == '"') or (s[0] == "'" and s[-1] == "'")):
        return s[1:-1]

    return s.strip("\"'")


def _parse_http_datetime(value: str | None) -> _dt.datetime | None:
    if not value:
        return None
    try:
        parsed = email.utils.parsedate_to_datetime(value)
    except (TypeError, ValueError):
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=_dt.timezone.utc)
    return parsed.astimezone(_dt.timezone.utc)


def _iso_utc(dt: _dt.datetime) -> str:
    return dt.astimezone(_dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")



@dataclass(frozen=True)
class DownloadMeta:
    updated_utc: str
    etag: str
    sha256: str
    length: int


def _download_to_gz(url: str, gz_path: Path) -> DownloadMeta:
    fetched_at = _dt.datetime.now(tz=_dt.timezone.utc)

    req = urllib.request.Request(
        url,
        headers={
            "User-Agent": USER_AGENT,
            "Accept": "text/plain,*/*;q=0.9",
            "Accept-Encoding": "gzip",
        },
    )

    try:
        resp = urllib.request.urlopen(req, timeout=TIMEOUT_SECS)
    except urllib.error.URLError as e:
        raise SystemExit(f"Failed to download {url}: {e}")

    date_raw = resp.headers.get("Date") or ""
    date_dt = _parse_http_datetime(date_raw)
    server_date_utc = _iso_utc(date_dt) if date_dt else ""

    etag = _normalize_etag(resp.headers.get("ETag"))
    content_encoding = (resp.headers.get("Content-Encoding") or "").lower()

    sha = hashlib.sha256()
    total_bytes = 0

    tmp_path = gz_path.with_suffix(gz_path.suffix + ".tmp")
    tmp_path.parent.mkdir(parents=True, exist_ok=True)

    with gzip.open(tmp_path, mode="wb", compresslevel=9) as out:
        stream = resp
        if content_encoding == "gzip":
            stream = gzip.GzipFile(fileobj=resp)
        while True:
            chunk = stream.read(1024 * 1024)
            if not chunk:
                break
            total_bytes += len(chunk)
            sha.update(chunk)
            out.write(chunk)

    tmp_path.replace(gz_path)

    return DownloadMeta(
        updated_utc=_iso_utc(fetched_at),
        etag=etag,
        sha256=sha.hexdigest(),
        length=total_bytes,
    )


def _write_meta(meta_path: Path, meta: DownloadMeta) -> None:
    meta_path.write_text(
        json.dumps(asdict(meta), indent=2, sort_keys=True, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def _normalize_meta_files_in_place() -> int:
    updated = 0
    meta_files = sorted(Path(".").glob("*.meta"))
    for meta_path in meta_files:
        try:
            raw = meta_path.read_text(encoding="utf-8")
        except OSError:
            continue

        try:
            parsed = json.loads(raw)
        except json.JSONDecodeError:
            # Not JSON (e.g. domains.txt.meta) — ignore.
            continue

        if not isinstance(parsed, dict) or "etag" not in parsed:
            continue

        old = parsed.get("etag")
        new = _normalize_etag(old if isinstance(old, str) else "")
        if new == (old or ""):
            continue

        parsed["etag"] = new
        meta_path.write_text(
            json.dumps(parsed, indent=2, sort_keys=True, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        print(f"Normalized ETag in {meta_path}")
        updated += 1

    if updated == 0:
        print("No ETags needed normalization")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--normalize-meta",
        action="store_true",
        help="Normalize ETag formatting in existing JSON .meta files without downloading",
    )
    args = parser.parse_args()

    if args.normalize_meta:
        return _normalize_meta_files_in_place()

    wrote = 0
    for filename, url in SOURCES:
        gz_path = Path(filename + ".gz")
        meta_path = Path(filename + ".meta")

        meta = _download_to_gz(url, gz_path)
        _write_meta(meta_path, meta)

        print(
            f"Wrote {gz_path} ({meta.length} bytes source, "
            f"updated_utc={meta.updated_utc}"
        )
        print(f"Wrote {meta_path}")
        wrote += 1

    return 0 if wrote else 1


if __name__ == "__main__":
    raise SystemExit(main())
