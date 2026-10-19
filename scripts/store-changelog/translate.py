#!/usr/bin/env python3
"""Translate the English store copy into every locale and write the fastlane files.

Reads:
    scripts/store-changelog/out/android.en.txt   (one bullet per line)
    scripts/store-changelog/out/apple.en.txt
    scripts/store-changelog/locales.json
    <version_code_source> (Constants.kt) for the Android versionCode

Writes (paths from locales.json, relative to repo root / cwd):
    Android: fastlane/metadata/android/<android>/changelogs/<versionCode>.txt   (plain lines, blank-separated)
    Apple:   nym-vpn-apple/fastlane/metadata/<apple>/release_notes.txt

Translation: DeepL Pro, one request per (locale, platform), each bullet sent as a separate
text so structure is preserved. German uses informal "du" (formality=prefer_less).

Auto-fit (never fails): if any Android locale (incl. en-US) exceeds the char limit, drop the
lowest-ranked (last) English bullet and re-translate ALL locales. This keeps every locale
showing the same set of bullets. Repeats until everything fits or one bullet remains.

Env: DEEPL_AUTH_KEY (required).
"""
import json
import os
import re
import sys
import urllib.parse
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "out")
CONFIG = json.load(open(os.path.join(HERE, "locales.json"), encoding="utf-8"))
DEEPL_URL = "https://api.deepl.com/v2/translate"
KEY = os.environ["DEEPL_AUTH_KEY"]


def read_bullets(name):
    path = os.path.join(OUT, name)
    with open(path, encoding="utf-8") as f:
        return [l.strip() for l in f if l.strip()]


def version_code():
    src = CONFIG["version_code_source"]
    text = open(src, encoding="utf-8").read()
    m = re.search(r"VERSION_CODE\s*=\s*(\d+)", text)
    if not m:
        sys.exit(f"Could not find VERSION_CODE in {src}")
    return m.group(1)


def deepl(bullets, target, formality=None):
    data = [("text", b) for b in bullets]
    data += [("source_lang", "EN"), ("target_lang", target)]
    if formality:
        data.append(("formality", formality))
    req = urllib.request.Request(
        DEEPL_URL,
        data=urllib.parse.urlencode(data).encode("utf-8"),
        headers={
            "Authorization": f"DeepL-Auth-Key {KEY}",
            "Content-Type": "application/x-www-form-urlencoded",
        },
    )
    with urllib.request.urlopen(req, timeout=60) as resp:
        out = json.load(resp)
    return [t["text"] for t in out["translations"]]


def join(bullets):
    return "\n\n".join(bullets)


def render_all(bullets, platform):
    """Return {locale: text} for every locale that targets `platform`."""
    rendered = {}
    for loc, cfg in CONFIG["locales"].items():
        if cfg.get(platform) is None:
            continue
        if cfg["deepl"] is None:  # source locale
            rendered[loc] = join(bullets)
        else:
            rendered[loc] = join(deepl(bullets, cfg["deepl"], cfg.get("formality")))
    return rendered


def write_files(rendered, platform, version):
    tmpl = CONFIG[f"{platform}_changelog_path" if platform == "android"
                  else "apple_release_notes_path"]
    for loc, text in rendered.items():
        sub = CONFIG["locales"][loc][platform]
        path = tmpl.format(android=sub, apple=sub, version_code=version)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            f.write(text + "\n")
        print(f"  {platform} {loc:7} {len(text):4} -> {path}")


def main():
    version = version_code()
    limit = CONFIG["android_char_limit"]

    # --- Android with auto-fit ---
    bullets = read_bullets("android.en.txt")
    while True:
        rendered = render_all(bullets, "android")
        longest = max(len(t) for t in rendered.values())
        if longest <= limit or len(bullets) <= 1:
            break
        dropped = bullets.pop()
        print(f"[auto-fit] longest={longest} > {limit}; dropped bullet: {dropped!r}")
    over = {loc: len(t) for loc, t in rendered.items() if len(t) > limit}
    print(f"Android versionCode {version} — {len(rendered)} locales (limit {limit}):")
    write_files(rendered, "android", version)
    if over:
        print(f"::warning:: Android still over limit after dropping to {len(bullets)} "
              f"bullet(s): {over}")

    # --- Apple (no hard limit) ---
    apple_bullets = read_bullets("apple.en.txt")
    print("Apple release_notes:")
    write_files(render_all(apple_bullets, "apple"), "apple", version)


if __name__ == "__main__":
    main()
