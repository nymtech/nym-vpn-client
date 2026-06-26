---
name: openlore-implement-story
description: Implement a story on a brownfield codebase using openlore structural context. Runs orient + risk check before coding, validates against specs, enforces a test gate before drift check.
license: MIT
compatibility: openlore MCP server
---

# openlore: Implement Story

## When to use this skill

Trigger this skill when the user asks to **implement a story or task** on a codebase that has
openlore analysis available, with phrasings like:
- "implement story X"
- "work on task Y"
- "start implementing this feature"
- explicit command `/openlore-implement-story`

**Prerequisite**: openlore analysis must exist (`openlore analyze` has been run).
If `orient` returns `"error": "no cache"` → run `analyze_codebase` first, then retry.

---

## Step 1 — Read the story and risk context

Read the story file. Extract:
- `$STORY_TITLE`, `$AC` (acceptance criteria), `$PROJECT_ROOT`
- `$RISK_CONTEXT` — the `risk_context` section if present (pre-filled by Architect Agent)

| Situation | Approach |
|---|---|
| `risk_context` present, risk 🟢 < 40 | Skip to Step 3 — use insertion point from context |
| `risk_context` present, risk 🟡 40–69 | Run Step 2 impact check, then proceed |
| `risk_context` present, risk 🔴 ≥ 70 | Stop — a blocking refactor story must be resolved first |
| `risk_context` absent | Run the full Step 2 orientation |

---

## Step 2 — Orient and assess risk

Call the openlore MCP tool `orient` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "task": "$STORY_TITLE",
  "limit": 7
}
```

For the top 2 functions returned, get minimal context first (callers, callees, body, test coverage in one call):
```json
// get_minimal_context
{
  "directory": "$PROJECT_ROOT",
  "functionName": "$FUNCTION_NAME"
}
```

**What to read from the result before proceeding:**
- `function.riskLevel` — `"high"` means fanIn ≥ 30 or fanOut ≥ 15; the tool expanded caller/callee lists to 24. All shown entries are in scope.
- `callers[*].callType` — all `"awaited"` = async interface frozen; changing signature or return type breaks every caller. Mixed = looser coupling.
- `callees[*].isExternal: true` — function touches HTTP/DB boundary; new code paths may fail silently in tests (mocked) but loudly in production.
- `testedBy[*].confidence` — `"called"` = direct test (strong). `"imported"` = test imports module only; `vi.mock()` can nullify it. Only `"imported"` entries = treat as effectively untested.

If `riskLevel` is `"high"` or any callee is external, check the cluster:
```json
// get_cluster
{
  "directory": "$PROJECT_ROOT",
  "functionName": "$FUNCTION_NAME"
}
```
- `clusterDensity < 0.05` → sparse, change is isolated, proceed
- `clusterDensity 0.05–0.15` → check `internalCallGraph` for transitively dependent functions
- `clusterDensity > 0.15` → dense cluster; coordinate the whole cluster or discuss scope with user

Then check risk:
```json
// analyze_impact
{
  "directory": "$PROJECT_ROOT",
  "symbol": "$FUNCTION_NAME",
  "depth": 2
}
```

**If any function has `riskScore ≥ 70`: stop.**
Do not implement. Run `/openlore-execute-refactor` on the blocking function first, or create a
blocking refactor task and return to this story once the risk is resolved.

---

## Step 2.5 — Stack inventory (conditional)

Based on the story title and orient results, call the relevant inventory tool(s) **before reading any source file**. Skip if the story clearly involves none of these areas.

| Story involves | Tool | Purpose |
|---|---|---|
| Data models / ORM / database / tables | `get_schema_inventory` | See existing tables and fields — don't re-invent what already exists |
| HTTP routes / API / endpoints | `get_route_inventory` | See existing routes before adding new ones |
| Config / env vars / secrets | `get_env_vars` | Identify which vars are required vs have defaults |
| UI components | `get_ui_components` | See existing component props and framework |

Call whichever openlore MCP inventory tool applies, e.g. `get_schema_inventory` with `{"directory": "$PROJECT_ROOT"}`.

Use the results to ground the implementation in existing schemas/routes — the plan cannot contradict what already exists.

---

## Step 3 — Check the spec

First, verify that OpenSpec specs exist:

```bash
ls $PROJECT_ROOT/openspec/specs/ 2>/dev/null | wc -l
```

**If 0 specs found:**
> No OpenSpec specs exist yet. `search_specs` will return empty results and
> `check_spec_drift` (Step 7) will flag everything as uncovered.
>
> Recommended: run `/openlore-generate` after this story to create a spec baseline.
> You only need to do this once.
>
> Continuing with structural analysis only.

Skip the `search_specs` call and go to Step 4.

**If specs exist:**

Call the openlore MCP tool `search_specs` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "query": "$STORY_TITLE",
  "limit": 5
}
```

If relevant requirements are found, read the domain spec before writing any code.
Note any constraints that apply.

---

## Step 3.5 — Audit spec coverage of the target domain

Call the openlore MCP tool `audit_spec_coverage` with `{"directory": "$PROJECT_ROOT"}`.

From the result, check:
- `staleDomains` — if the target domain appears here, its spec is outdated.
  Recommend running `openlore generate --domains $DOMAIN` before implementing.
- `hubGaps` — uncovered hub functions. If the feature touches one of these,
  add it to the adversarial check in Step 4b (high blast radius + no spec = risk).

If both are clean, continue to Step 4 without action.

---

## Step 4 — Find the insertion point

Use `insertion_points` from `risk_context` if present. Otherwise:

Call the openlore MCP tool `suggest_insertion_points` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "description": "$STORY_TITLE",
  "limit": 5
}
```

Read the skeleton of the target file by calling the openlore MCP tool `get_function_skeleton` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "filePath": "$TARGET_FILE"
}
```

**Confirm the approach with the user before writing code.**

### Step 4b — Adversarial challenge

Before writing any code, state explicitly what could break with this approach.
If `.claude/antipatterns.md` exists, read it and include any applicable patterns.

> "Risk check on `$INSERTION_POINT`:
> - `$CALLER_A` and `$CALLER_B` depend on this function — verify their assumptions
>   hold after the change.
> - `$EDGE_CASE` is not covered by the current test suite — add it in Step 6.
> - [if antipatterns apply] AP-NNN (`$PATTERN_NAME`) — `$RULE` — applies here because `$REASON`."

This is not a gate — do not wait for user input. It is a mandatory self-check
that must appear in the output before the first line of code is written.

---

## Step 5 — Implement

Apply changes in this order:
1. New types/interfaces (if needed)
2. Core logic at the insertion point
3. Updated call sites (if any)

Do not touch functions outside the scope identified in Step 2 / `risk_context` without
re-running the gate.

**Small model constraint**: if the model is under 13B parameters, each edit must touch a contiguous block of at most 50 lines. Split larger changes.

---

## Step 6 — Tests

Both levels required before proceeding to Step 7.

**Mandatory — existing tests must not regress:**
Run the full test suite. If any pre-existing test breaks, fix the regression before continuing.

**Recommended — at least one new test per AC:**
Write a test that directly exercises the behaviour described in the acceptance criterion.

| Situation | Action |
|---|---|
| All tests green, new tests written | Proceed to Step 7 |
| Existing test broken | Fix regression. Do not proceed. |
| New test reveals a misunderstanding of the AC | Return to Step 5, adjust implementation |
| Brownfield: no existing test coverage | Write the new test anyway. Note the coverage gap. |

---

## Step 7 — Verify drift

Only run once tests are green.

Call the openlore MCP tool `check_spec_drift` with `{"directory": "$PROJECT_ROOT"}`.

| Drift type | Resolution |
|---|---|
| `uncovered` on new files | Note it — propose `openlore generate` post-sprint |
| `gap` on existing domain | Run `openlore generate --domains $DOMAIN` |
| `stale` | Fix the reference |
| No drift | Done |

---

## Absolute constraints

- Do not write code before Step 4 confirmation
- If `riskScore ≥ 70` — stop, do not work around it, run `/openlore-execute-refactor` first
- Do not run `check_spec_drift` before tests are green
- Do not propose a spec update on untested code
