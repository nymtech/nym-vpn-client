---
name: openlore-debug
description: Debug a problem by anchoring root-cause analysis in openlore structural knowledge. Uses orient + search_specs + analyze_impact to form an explicit hypothesis before reading code. Enforces RED/GREEN test verification.
license: MIT
compatibility: openlore MCP server
---

# openlore: Debug

## When to use this skill

Trigger this skill when the user reports **a bug or unexpected behaviour** on a codebase
that has openlore analysis available, with phrasings like:
- "this is broken"
- "X is not working"
- "something is wrong with Y"
- "debug this"
- explicit command `/openlore-debug`

**The rule**: form an explicit hypothesis before reading any code. Do not browse
files speculatively.

**Prerequisite**: openlore analysis must exist (`openlore analyze` has been run).
If `orient` returns `"error": "no cache"` → run `analyze_codebase` first, then retry.

---

## Step 1 — Reproduce

Ask the user for:
1. **Steps to reproduce** — minimal sequence that triggers the bug
2. **Expected behaviour** — what should happen
3. **Observed behaviour** — what actually happens
4. **`$PROJECT_ROOT`** — project root directory

Do not proceed to Step 2 until all four are known.

If the user cannot reproduce the bug reliably, note it and proceed anyway — but
flag that the fix may be speculative until reproduction is confirmed.

Capture:
- `$BUG_DESCRIPTION` — one-line summary of the symptom (e.g. "payment retry does not reset counter after success")
- `$REPRO_STEPS` — reproduction sequence

---

## Step 2 — Orient

Call the openlore MCP tool `orient` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "task": "$BUG_DESCRIPTION",
  "limit": 7
}
```

Extract:
- **`$CANDIDATE_FUNCTIONS`** — top 3–5 functions structurally related to the symptom
- **`$DOMAINS_AFFECTED`** — spec domains involved
- **`$CALL_PATHS`** — call chains relevant to the symptom

---

## Step 3 — Search specs

If `openspec/specs/` exists:

Call the openlore MCP tool `search_specs` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "query": "$BUG_DESCRIPTION",
  "limit": 5
}
```

Look for:
- **Documented constraints** that the buggy behaviour violates
- **Requirements** that define what "correct" means for `$DOMAINS_AFFECTED`
- **Known edge cases** documented in the spec that may explain the symptom

If no specs exist, skip this step and note the absence.

---

## Step 4 — Isolate and hypothesize

For the top 2 candidate functions from Step 2, get minimal context first (callers, callees, body, test coverage in one call):
```json
// get_minimal_context
{
  "directory": "$PROJECT_ROOT",
  "functionName": "$CANDIDATE_FUNCTION"
}
```

Then check structural properties:
```json
// analyze_impact
{
  "directory": "$PROJECT_ROOT",
  "symbol": "$CANDIDATE_FUNCTION",
  "depth": 2
}
```

If the repro involves a request flow (HTTP request, event, message queue), confirm the call chain before forming the hypothesis by calling the openlore MCP tool `trace_execution_path` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "from": "$ENTRY_POINT",
  "to": "$CANDIDATE_FUNCTION"
}
```

This replaces speculative file browsing — the path is structural fact, not inference. Skip if `$ENTRY_POINT` is unknown or the repro is not request-driven.

Using the call paths, risk scores, spec constraints, and traced path gathered so far,
**state an explicit hypothesis before reading any code**:

> "Hypothesis: `$FUNCTION` does not reset `$STATE` when `$CONDITION` because
> it is called from `$CALLER` which does not pass `$PARAMETER`."

The hypothesis must:
- Name a specific function
- State a specific mechanism (state not reset, condition not checked, wrong caller, etc.)
- Be falsifiable by reading the code

**Do not read source files before forming this hypothesis.**

---

## Step 5 — Verify the hypothesis

Read the skeleton of the hypothesised function(s) by calling the openlore MCP tool `get_function_skeleton` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "filePath": "$TARGET_FILE"
}
```

Then read the full function body if needed.

| Result | Action |
|---|---|
| Hypothesis confirmed | Proceed to Step 6 |
| Hypothesis refuted | Return to Step 4 with a revised hypothesis (max 3 iterations before asking the user for more context) |
| Cause is in a caller, not the function itself | Extend `analyze_impact` one level up, revise hypothesis |

Document the confirmed hypothesis explicitly before writing any fix.

---

## Step 6 — Fix

Apply the **minimal fix** that resolves the confirmed hypothesis.

Do not modify functions outside the scope identified in Steps 2–5 without
re-running the gate (`orient` + `analyze_impact`) on the new scope.

**Small model constraint**: each edit must touch a contiguous block of at most
50 lines. Split larger fixes into sequential edits.

Do not refactor, rename, or clean up unrelated code while fixing the bug.

---

## Step 7 — Verify

**RED first (if no existing test covers this case):**

Write a test that reproduces the bug using `$REPRO_STEPS`. Run it. It must fail
(RED) — confirming the bug is real and the test is meaningful.

**Apply the fix**, then run the test again. It must pass (GREEN).

**Full suite:**

Run the full test suite. If any pre-existing test breaks, fix the regression
before closing the bug.

| Situation | Action |
|---|---|
| New test RED → fix → GREEN, suite green | Proceed to Step 8 |
| Cannot reproduce in a test | Note it, apply fix, confirm manually, add a note in the story/issue |
| Suite regression introduced | Fix regression. Do not proceed. |

---

## Step 8 — Drift check

Only if the fix changes a documented behaviour:

Call the openlore MCP tool `check_spec_drift` with `{"directory": "$PROJECT_ROOT"}`.

| Drift type | Resolution |
|---|---|
| `gap` on modified function | The spec described expected behaviour that changed — update the spec |
| `stale` | Fix the stale reference |
| `uncovered` | Not caused by this fix — note it, propose `openlore generate` |
| No drift | Proceed to Step 9 |

---

## Step 9 — Spec invariant feedback

Every real bug reveals an invariant that was not documented. Capture it so future
agents benefit from this discovery via `search_specs`.

**9a — Identify the invariant**

State the invariant that was violated, in one sentence:

> "`$FUNCTION` must always `$CONDITION` when `$TRIGGER` — violating this causes
> `$OBSERVED_SYMPTOM`."

If the bug was caused by a missing guard, a wrong assumption about caller order,
or an undocumented state constraint — that is the invariant.

**9b — Locate the spec**

Call the openlore MCP tool `get_spec` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "domain": "$DOMAIN_AFFECTED"
}
```

**9c — Add the invariant**

Append to the relevant domain spec under a `### Known Invariants` section
(create it if absent). Wrap the section in `<!-- manual -->` / `<!-- /manual -->`
markers so `openlore generate` preserves it on re-generation:

```markdown
<!-- manual -->
### Known Invariants

- `$FUNCTION`: $INVARIANT_STATEMENT
  — discovered via bug fix on $DATE, root cause: $ROOT_CAUSE_SUMMARY
<!-- /manual -->
```

If the domain spec does not exist yet (`uncovered` from Step 8), note the
invariant in the story/issue instead and flag it for the next `openlore generate` run.

**9d — Evaluate cross-cutting scope**

Ask: is this bug an instance of a general failure pattern, or specific to this domain?

| Signal | Cross-cutting antipattern? |
|---|---|
| Bug involves an assumption about external state, ordering, or caller contract | Yes |
| Bug is reproducible in other domains with the same pattern | Yes |
| Bug is specific to a data invariant in `$DOMAIN` | No — domain spec only |

If cross-cutting, append to `.claude/antipatterns.md` (if absent, create it with the
header from the antipatterns template):

```markdown
## AP-{NNN} — {pattern name}

- **Class**: {state | concurrency | boundary | assumption | resource | ordering}
- **Symptom**: {what broke — one sentence}
- **Rule**: {detection rule — "When X, always verify Y"}
- **Discovered**: $DATE via $BUG_DESCRIPTION
```

**9e — Inform the user**

> "Invariant added to `openspec/specs/$DOMAIN/spec.md`."

If an antipattern was added:
> "Cross-cutting antipattern AP-{NNN} added to `.claude/antipatterns.md`.
> Future brainstorm and implementation sessions will check this rule."

---

## Absolute constraints

- Do not read source code before forming a hypothesis in Step 4
- Hypothesis is mandatory — even when the cause seems obvious
- Do not skip Step 1 (reproduction) — a fix without reproduction is speculation
- Do not touch functions outside the confirmed scope without re-running the gate
- Do not run `check_spec_drift` before tests are green
- Each edit ≤ 50 lines on small models
- Do not skip Step 9 — every bug fix must produce a documented invariant or an
  explicit note explaining why no invariant applies
