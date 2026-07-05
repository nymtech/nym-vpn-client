---
name: openlore-orient
version: 2.1
description: Persistent architectural memory for this codebase. Call `orient(task)` before reading source files to get the relevant functions, callers, spec sections, and insertion points for any task — one structural lookup instead of file-by-file rediscovery.
---

# OpenLore — orient before you read

This project uses **OpenLore** to maintain a deterministic, graph-native model of the codebase: every function, every caller, every spec section, every file. The `orient` tool collapses what would otherwise be a chain of `analyze_codebase → search_code → search_specs → suggest_insertion_points` into a single call.

The single most important habit when working in this repo: **call `orient` before opening source files for any non-trivial task.**

## When to use this skill

Call `orient` at the start of any of these:

- **New task in the repo** — even ones that feel small. The graph knows things the file tree doesn't.
- **Unknown function or symbol** — instead of grepping blind, ask `orient` and let it return the function, its callers, and the spec section that owns it.
- **Planning a cross-module change** — `orient` returns insertion points ranked by structural fit, so you start at the right boundary instead of the most-recently-viewed file.
- **After an Epistemic Lease prefix appears** — when a tool response says you're stale, re-orient before continuing. Acting on stale context is the most common failure mode.

If you are reading source files without having called `orient` first, you are probably wasting tokens.

## How to use it

### Preferred: via the OpenLore MCP server

If `openlore` is registered as an MCP server in this project (look for `.mcp.json` → `mcpServers.openlore` — the project-scope file Claude Code reads; **not** `.claude/settings.json`, which it ignores for MCP), call the `orient` tool directly through the MCP interface. This is the lowest-latency path and gives the model access to whatever tool surface the server was wired with — the lean 10-tool `navigation` preset by default (`openlore install --preset full` wires all 62), not just `orient`.

### Fallback: via the shell wrapper

```sh
bash scripts/orient.sh "<task description>"
```

(On Windows: `powershell -File scripts/orient.ps1 "<task description>"`)

The wrapper tries the direct CLI subcommand first (`npx --yes openlore orient --json --task "<task>"`) and falls back to driving the `openlore mcp` server over stdio JSON-RPC via the sibling `orient-via-mcp.mjs` helper on older openlore versions that predate the CLI subcommand. Either path produces real orient JSON on stdout. Parse these arrays from the result:

The result fields are **camelCase** (matching `orient --json`):

- **`relevantFunctions`** — top scored functions for the task, each with `name`/`file`/`line`, role classification, and a short reason.
- **`callPaths`** — caller/callee neighbourhood for the top functions. This is where "who breaks if I change this" lives.
- **`specDomains`** — OpenSpec domains that own the relevant files, including the requirements they encode. Read these before reading code.
- **`insertionPoints`** — ranked candidate locations to make the change, with structural justification.
- **`relevantFiles`**, **`provenance`**, **`changeCoupling`**, **`suggestedTools`**, **`nextSteps`** — supporting context. **`searchMode`** is `semantic` when embeddings are built or `bm25_fallback` otherwise.

Always start by reading **`specDomains`**, then **`callPaths`**, then jump into source only at the **`insertionPoints`** you actually plan to edit.

### Lean mode for shallow lookups (cheaper)

For a quick "who calls X" / "where is Y defined" lookup, pass **`lean: true`** (or `orient --lean`): it returns just the navigation core (`relevantFunctions` with `expand` handles, `callPaths`, `specDomains`) — ~40% smaller — and drops the provenance / change-coupling / insertion-points / specs / decisions enrichment, each still one `expand` handle or dedicated tool call away. Omit `lean` when you actually need that enrichment (planning an edit, checking specs/decisions). Caveat (measured, Spec 27): on a *trivial* lookup in a small/familiar repo, even a lean orient call can cost more than just grepping — `orient` earns its keep on unfamiliar or multi-hop work, not one-line facts you could find in 2 reads.

> **Note:** the `openlore orient --json --task` CLI subcommand is available, so the wrappers use it directly. The MCP fallback in the wrappers only kicks in on older openlore versions that predate the subcommand.

> **Want to measure it on your own repo?** Pass `--metrics` (off by default) to have `orient` report its wall time and output size to stderr. Nothing is measured or printed unless you ask for it.

## What NOT to do

- **Do not open source files before `orient` has returned.** This is the single most expensive mistake — you'll re-derive what the graph already knows.
- **Do not call `orient` on every edit.** It's a session-start and re-orient-on-staleness tool, not a per-call helper. Respect the Epistemic Lease signal — if no prefix appears, your context is still fresh.
- **Do not paraphrase the task** when passing it to `orient`. Use the user's words. The semantic search matches better when the query language matches the eventual prompt.
- **Do not ignore `specDomains`.** Reading specs first is faster and more accurate than reading code first, in this repo.

## Failure modes

If `orient` returns an empty result or errors:

1. **Empty `relevantFunctions` array** — the task description didn't match the graph. Try rephrasing using a known module name, function name, or domain word from the spec. Do not fall back silently — tell the user that `orient` didn't match and you're going to grep.
2. **Wrapper output contains `"error": "No analysis found"`** — the codebase hasn't been analyzed yet. Run `npx openlore analyze` once to seed the graph, then retry. The wrapper itself is healthy in this case; the underlying tool is telling you what's missing.
3. **Graph is stale (mtime older than recent edits)** — the JSON output will still come back, but the insertion points may be wrong. Re-run `npx openlore analyze` to rebuild.

In all failure cases, the correct fallback is a *targeted* `grep`/file read scoped to a single module — not opening the whole repo. And **always tell the user the skill silently degraded** so they can rebuild the graph if needed.
