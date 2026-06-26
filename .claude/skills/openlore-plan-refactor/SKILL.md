---
name: openlore-plan-refactor
description: Identify the highest-priority refactoring target using static analysis, assess its blast radius, and produce a detailed written plan saved to .openlore/refactor-plan.md. Makes no code changes.
license: MIT
compatibility: openlore MCP server
---

# openlore: Plan Refactor

## When to use this skill

Trigger this skill whenever the user asks to **plan a refactoring** on a codebase, with phrasings like:
- "plan a refactoring of X"
- "analyze my code and prepare a refactor plan"
- "generate a refactoring plan"
- "I want to refactor this function / this file"
- explicit command `/openlore-plan-refactor`

**This skill modifies no code files.** It only produces `.openlore/refactor-plan.md`.
To apply the plan, use the `openlore-execute-refactor` skill.

---

## ⚠️ Fundamental principle — each change is a complete mini-development

**The plan must be written so that every entry in the change sequence is independently executable and testable.**

Each change = one atomic edit → one diff verification → one full test run → ✅ or rollback.

This is not a final gate at the end. Testing is a mandatory sub-step after **every single change**.
The plan must make this cycle explicit and impossible to skip.

**For small models (< 13B parameters)**: each change must touch at most **50 contiguous lines** in the source file.
If a logical extraction exceeds this limit, split it into smaller sub-changes, each with its own test gate.

---

## User-specified target — shortcut path

If the user has already named a specific file or function to refactor:
- **Skip** Steps 2, 3, and 3b (discovery is not needed).
- **Do not skip** Steps 3c, 4, 5, 6, and 6b — coverage check and impact analysis are mandatory regardless of how the target was chosen.
- Jump directly to Step 3c using the user-provided target.

---

## Step 1 — Confirm the project directory

Ask the user which project to analyze, or confirm the current workspace root.

---

## Step 2 — Run static analysis

Call the openlore MCP tool `analyze_codebase` with `{"directory": "$DIRECTORY"}`.

If a recent analysis already exists, skip unless the user explicitly requests a fresh run.

---

## Step 3 — Get the refactoring report

Call the openlore MCP tool `get_refactor_report` with `{"directory": "$DIRECTORY"}`.

Present the top 5 candidates:

| Function | File | Issues | Priority score |
|---|---|---|---|

---

## Step 3b — Check for duplicate code

Call the openlore MCP tool `get_duplicate_report` with `{"directory": "$DIRECTORY"}`.

If a top candidate appears in a clone group, prepend a **deduplication note** to the plan:
> "⚠️ `<function>` has N near-clones. Consolidate them first to reduce the blast radius of this refactor."

---

## Step 3c — Check test coverage

Before presenting a choice to the user, check test coverage for the files containing the candidates. Detect the coverage tool from the project:

| Ecosystem | Command |
|---|---|
| Node.js | `npm test -- --coverage --collectCoverageFrom="<files>"` |
| Python | `pytest --cov=<module> --cov-report=term-missing` |
| Rust | `cargo tarpaulin --include-files <files>` |
| Go | `go test -cover ./...` |

Enrich the candidate table:

| Function | File | Priority | Coverage |
|---|---|---|---|
| ... | ... | ... | 72% ✅ / 35% ⚠️ / 0% 🚫 |

**Thresholds:**

| Coverage | Badge | Meaning |
|---|---|---|
| ≥ 70% lines | ✅ | Safe to refactor |
| 40–69% lines | ⚠️ | Write characterisation tests first |
| < 40% lines | 🛑 | Strongly discouraged |
| 0% (no tests) | 🚫 | Blocked — propose a test harness first |

If **all candidates are below 40%**:
> "Every high-priority target has insufficient test coverage (< 40%). I recommend writing a minimal test harness for at least one target before proceeding. Would you like me to suggest test cases based on the function signatures?"

---

## Step 4 — Get minimal context + analyze impact

Get the condensed view of the target function (callers, callees, body, test coverage in one call):

Call the openlore MCP tool `get_minimal_context` with `{"directory": "$DIRECTORY", "functionName": "$FUNCTION_NAME"}`.

**What to read before deciding whether to proceed:**
- `function.riskLevel` — `"high"` (fanIn ≥ 30 or fanOut ≥ 15) means up to 24 callers/callees shown. Read all — they are all in scope for this refactor.
- `callers[*].callType` — all `"awaited"` = async interface frozen; extracting or splitting requires updating every call site. `"direct"` / `"method"` = more portable.
- `testedBy[*].confidence` — `"called"` = direct test, safe to refactor under. `"imported"` only = `vi.mock()` can neutralize it; write a characterisation test **before** refactoring.

Then get the full impact analysis:

Call the openlore MCP tool `analyze_impact` with `{"directory": "$DIRECTORY", "symbol": "$FUNCTION_NAME"}`.

Note: risk score (0–100), recommended strategy (`extract` / `split` / `facade` / `delegate`), top 5 upstream callers and downstream callees.

**If `riskLevel` is `"high"` or impact risk score ≥ 60**, check the cluster to set refactor scope:

Call the openlore MCP tool `get_cluster` with `{"directory": "$DIRECTORY", "functionName": "$FUNCTION_NAME"}`.

Read `stats.clusterDensity`:
- `< 0.05` — sparse; extract the target independently, others not in scope
- `0.05–0.15` — moderate; include functions in `internalCallGraph` that call the target in the plan's risk section
- `> 0.15` — dense; refactor the whole cluster together or not at all; add all members to affected-files list

---

## Step 5 — Visualise the call neighbourhood

Call the openlore MCP tool `get_subgraph` with:
```json
{"directory": "$DIRECTORY", "functionName": "$FUNCTION_NAME", "direction": "both", "format": "mermaid"}
```

Show the Mermaid diagram to the user.

---

## Step 6 — Find safe entry points (bottom-up)

Call the openlore MCP tool `get_low_risk_refactor_candidates` with:
```json
{"directory": "$DIRECTORY", "filePattern": "$TARGET_FILE", "limit": 5}
```

Cross-reference with the subgraph from Step 5: a good first extraction candidate already appears as a callee of the target function.

---

## Step 6b — Find insertion points for extracted helpers

Before designing the change sequence, identify where extracted functions should land.
This avoids creating helpers in the wrong file or layer.

Call the openlore MCP tool `suggest_insertion_points` with:
```json
{"directory": "$DIRECTORY", "query": "extract helper from $FUNCTION_NAME", "limit": 5}
```

For each candidate, note its role and strategy. Prefer candidates that already call into — or are called by — the target function (visible in the Step 5 subgraph).

---

## Step 7 — Design the change sequence

Design an ordered sequence of atomic changes based on the strategy from Step 4.

### Size constraint (mandatory for small models)

**Each change must touch at most 50 contiguous lines** in the source file.
If a logical extraction requires moving more than 50 lines in one pass, split it into sub-changes:
- Sub-change A: extract the inner block (≤ 50 lines)
- Sub-change B: extract the outer wrapper (≤ 50 lines)
Each sub-change gets its own test gate. Never group two sub-changes before testing.

### Each change must specify

- **What**: the exact block to move (line range or description)
- **Lines touched**: estimated count in source file — must be ≤ 50
- **New name**: the function or method name to give it
- **Target file**: existing or new file (with justification)
- **Target class** (if applicable)
- **Call sites to update**: list each `file:line`
- **Test gate**: exact test command to run after this change

### Mini-development cycle per change (write this explicitly in the plan)

```
READ plan entry → EDIT (targeted, ≤ 50 lines) → DIFF verify → TEST
  ├─ green → mark ✅, next change
  └─ red   → git checkout HEAD -- <file>, diagnose, retry
             after 3 failed retries → STOP, report to user
```

The plan must show this cycle in the change sequence, not just in a footnote.

**Rules per strategy:**

| Strategy | Rule |
|---|---|
| `split` | Decompose into N sub-functions in the same file unless they clearly belong elsewhere |
| `extract` | Place in the nearest cohesive module or create a new file if none exists |
| `facade` | Keep the original signature, delegate to smaller functions; companion module if > 300 lines |
| `delegate` | Move ownership logic to callers; update every caller file in the upstream chain |

**Present the full sequence and wait for confirmation before writing the plan file.**

---

## Step 8 — Write `.openlore/refactor-plan.md`

Fill every section — **leave nothing as "TBD"**.

```markdown
# Refactor Plan

Generated: <ISO date>
Workflow: /openlore-plan-refactor → /openlore-execute-refactor

## Target
- **Function**: <n>
- **File**: <relative path>
- **Lines**: <start>–<end>
- **Risk score**: <0–100>
- **Strategy**: <extract | split | facade | delegate>
- **Priority score before refactor**: <value>

## Why
- <issue 1>
- <issue 2>

## Callers (upstream — must not break)
| Caller | File |
|---|---|

## Callees (downstream — candidates for extraction)
| Callee | File |
|---|---|

## Coverage baseline
- **File**: <target file>
- **Coverage**: <X>% lines, <Y>% branches
- **Status**: ✅ safe / ⚠️ caution / 🛑 discouraged
- **Test command**: <exact command>

## Change sequence
Each change is a complete mini-development: edit → diff → test → ✅ or rollback.
Never advance to the next change without a green test gate.

### Change 1 — <short label>
- **What**: extract lines <start>–<end> (logic: <one-line description>)
- **Lines touched in source**: ~<N> lines (must be ≤ 50)
- **New function name**: `<n>`
- **Target file**: `<path>` (<new file | existing file — reason>)
- **Target class**: `<ClassName>` or none
- **Call sites to update**: <list each file:line>
- **Expected diff**: +<N> lines in <target file>, -<M> lines in <source file>
- **Test gate**: `<exact test command>`
- **Retry limit**: 3 attempts — if still red after 3, stop and report

### Change 2 — <short label>
...

## Acceptance criteria
- Priority score drops below <target score> in `get_refactor_report`
- Function exits the top-5 list
- Full test suite passes (green)
- `git diff --stat` shows only the expected files

## Restore point
Hash: <to be filled by the execute workflow>
```

Once the file is written:
> "Plan written to `.openlore/refactor-plan.md`. Review it, then run `/openlore-execute-refactor` to apply the changes."

---

## Absolute constraints

- **No code modifications** in this workflow
- Always read the source file to confirm exact line numbers
- Never leave a section empty or as "TBD"
- Each change in the plan must include a test gate and a ≤ 50-line scope
- Prefer candidates with higher coverage when scores are otherwise close
- If the user does not pick a target, default to the top candidate
