---
name: openlore-analyze-codebase
description: Run a full static analysis of a project using openlore and summarise the results — architecture, call graph, top refactoring issues, and duplicate code. No LLM required.
license: MIT
compatibility: openlore MCP server
---

# openlore: Analyze Codebase

## When to use this skill

Trigger this skill whenever the user asks to **analyze a codebase** with openlore, with phrasings like:
- "analyze my project / my code"
- "give me a code quality report"
- "what are the structural issues in my codebase"
- "find duplicates in my code"
- explicit command `/openlore-analyze-codebase`

This skill is **read-only** — it modifies no files. It produces a report and suggests next steps.

---

## Step 1 — Confirm the project directory

Ask the user which project to analyze, or confirm the current workspace root.

```
Which project directory should I analyze?
Options: current workspace root | enter a different path
```

---

## Step 2 — Run static analysis

Call the openlore MCP tool `analyze_codebase` with `{"directory": "$DIRECTORY"}`.

---

## Step 3 — Summarize the results

Present a concise summary:
- Project type and detected frameworks
- File count, function count, internal call count
- Top 5 refactoring issues (function name, file, issue type, priority score)
- Detected domains

Also report stack inventory (read directly from `.openlore/analysis/` — no extra MCP call needed):
- **HTTP routes**: N routes across M files — if `route-inventory.json` exists
- **ORM tables**: N tables — if `schema-inventory.json` exists
- **Env vars**: N total, X required without default — if `env-inventory.json` exists
- **UI components**: N components — if `ui-inventory.json` exists

If none of these files exist, skip this section and suggest running `openlore analyze --force`.

---

## Step 4 — Show the call graph

Call the openlore MCP tool `get_call_graph` with `{"directory": "$DIRECTORY"}`.

Highlight:
- **Hub functions** (fanIn ≥ 8) — over-solicited functions, high coupling risk
- **Layer violations** detected (e.g. a UI layer calling the database directly)

---

## Step 5 — Show duplicate code report

Call the openlore MCP tool `get_duplicate_report` with `{"directory": "$DIRECTORY"}`.

Present a concise summary:
- Overall duplication ratio (e.g. "12% of functions are duplicated")
- Top 3 clone groups sorted by impact (instances × line count):
  - Clone type (exact / structural / near) and similarity score
  - List of instances (file + function name + line range)
- If no duplicates found, note this as a positive signal

---

## Step 6 — Suggest next steps

Based on the analysis, guide the user through the natural next actions in order:

1. Call `get_minimal_context` on the highest-priority function — returns callers, callees, body, and test coverage in one call (~300 tokens). Use instead of `get_subgraph` + `get_signatures` separately.
2. Call `get_cluster` on any function to see its full community (tightly coupled neighbors across directories).
3. Call `detect_changes` to rank recently changed functions by blast radius — spot riskiest commits before reviewing.
3. If significant duplication was found, suggest consolidating clone groups **before** refactoring
4. Suggest running `/openlore-plan-refactor` once the user has enough context to act, then `/openlore-execute-refactor` to apply the plan
5. If the project has OpenSpec specs, call `list_spec_domains` then `search_specs` to enable
   spec-first reasoning (question → requirements → linked source files). To activate `search_specs`,
   run `openlore analyze --embed` or `openlore analyze --reindex-specs`.

---

## Absolute constraints

- **No code modifications** in this workflow
- Never skip the duplication step — it determines the order of subsequent actions
- Always present call graph and duplicate report results even if numbers are low
- Next steps (Step 6) are suggestions, not automatic actions
