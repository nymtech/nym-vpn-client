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
GLOSSARY = json.load(open(os.path.join(HERE, "glossary.json"), encoding="utf-8"))
KEEP = GLOSSARY["keep_latin"]
# longest-first so multi-word terms mask before their constituents; a term starting with a
# non-word char (e.g. "$NYM") gets no leading word boundary.
KEEP_PATTERNS = [
    (t, re.compile(re.escape(t) if not t[0].isalnum() else rf"(?<!\w){re.escape(t)}(?!\w)"))
    for t in sorted(KEEP, key=len, reverse=True)
]
DEEPL_URL = "https://api.deepl.com/v2/translate"
KEY = os.environ["DEEPL_AUTH_KEY"]


def protect(texts):
    """Mask keep-Latin terms (brands/protocols) with shared sentinels so DeepL can't
    translate or transliterate them. Returns (masked_texts, sentinel->term map)."""
    mapping, term_to_sent = {}, {}

    def mask(t):
        for term, pat in KEEP_PATTERNS:
            if pat.search(t):
                sent = term_to_sent.get(term)
                if sent is None:
                    sent = f"⟦{len(term_to_sent)}⟧"  # ⟦N⟧ — survives DeepL intact
                    term_to_sent[term] = sent
                    mapping[sent] = term
                t = pat.sub(sent, t)
        return t

    return [mask(t) for t in texts], mapping


def restore(texts, mapping):
    """Put the verbatim terms back in place of their sentinels."""
    out = []
    for t in texts:
        for sent, term in mapping.items():
            t = t.replace(sent, term)
        out.append(t)
    return out


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
    if not bullets:
        return []  # DeepL 400s on a request with no `text` param — skip the call entirely
    masked, mapping = protect(bullets)
    data = [("text", b) for b in masked]
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
    return restore([t["text"] for t in out["translations"]], mapping)


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
    """Write each locale's file; return the list of paths written (for the generated-files manifest)."""
    tmpl = CONFIG[f"{platform}_changelog_path" if platform == "android"
                  else "apple_release_notes_path"]
    written = []
    for loc, text in rendered.items():
        if not text.strip():
            print(f"  {platform} {loc:7}    0 (empty — not written)")
            continue  # don't write/stage an empty store file when a platform has no bullets
        sub = CONFIG["locales"][loc][platform]
        path = tmpl.format(android=sub, apple=sub, version_code=version)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            f.write(text + "\n")
        print(f"  {platform} {loc:7} {len(text):4} -> {path}")
        written.append(path)
    return written


def main():
    version = version_code()
    limit = CONFIG["android_char_limit"]
    generated = []

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
    generated += write_files(rendered, "android", version)
    if over:
        print(f"::warning:: Android still over limit after dropping to {len(bullets)} "
              f"bullet(s): {over}")

    # --- Apple (no hard limit) ---
    apple_bullets = read_bullets("apple.en.txt")
    print("Apple release_notes:")
    generated += write_files(render_all(apple_bullets, "apple"), "apple", version)

    # Manifest of the files THIS run generated, so the length guard checks only these (not the whole
    # fastlane tree, which carries pre-existing over-limit legacy files from older releases).
    os.makedirs(OUT, exist_ok=True)
    with open(os.path.join(OUT, "generated.txt"), "w", encoding="utf-8") as f:
        f.write("\n".join(generated) + "\n")
    print(f"wrote manifest: {len(generated)} files -> out/generated.txt")


if __name__ == "__main__":
    main()
