---
name: openlore-execute-refactor
description: Apply the refactoring plan produced by openlore-plan-refactor. Reads .openlore/refactor-plan.md and re-reads it before each change to stay on track. Requires a confirmed plan to exist before running.
license: MIT
compatibility: openlore MCP server
---

# openlore: Execute Refactor

## When to use this skill

Trigger this skill when the user asks to **apply a refactoring plan**, with phrasings like:
- "apply the refactor plan"
- "execute the planned refactoring"
- explicit command `/openlore-execute-refactor`

**Prerequisite**: the `openlore-plan-refactor` skill must have been run and the plan confirmed.
The file `.openlore/refactor-plan.md` must exist.

---

## ⚠️ Fundamental principle — each change is a complete mini-development

Each change in the plan is treated as an **independent, self-contained development unit**.
The cycle for every single change, without exception:

```
READ plan entry
  → EDIT  (targeted tool, ≤ 50 lines touched)
  → DIFF  (git diff --stat + git diff — verify no lost code)
  → TEST  (run the test gate from the plan)
     ├─ green → mark ✅ in plan, move to next change
     └─ red   → git checkout HEAD -- <file>
                diagnose the failure
                retry from EDIT (attempt 2, then 3)
                if still red after 3 attempts → STOP (see circuit-breaker below)
```

**Never accumulate broken state. Never skip the test gate. Never batch two changes before testing.**

### Circuit-breaker — 3 failed attempts

If a change fails its test gate 3 times in a row, stop immediately and report:

> "Change N (`<label>`) failed after 3 attempts. The working tree has been restored to the last green state.
> Options:
> a) Split this change into smaller sub-changes (≤ 50 lines each) and update the plan
> b) Return to planning (`/openlore-plan-refactor`) to redesign this step
> c) Skip this change and continue — note the acceptance criteria may not be fully met"

Do not attempt a 4th retry without explicit user instruction.

---

## Step 1 — Read the plan

Read `.openlore/refactor-plan.md` from the project directory.

If the file does not exist, stop immediately:
> "No refactor plan found at `.openlore/refactor-plan.md`. Please run `/openlore-plan-refactor` first."

Extract and display a summary:
- Target function, file, and line range
- Strategy and risk score
- Number of changes planned
- Test command
- Acceptance criteria

**Ask the user to confirm before proceeding.**

> **Execution mode**: once confirmed, execute all changes in the plan **without asking for permission between steps**. The only valid reasons to pause mid-execution are: (a) a test fails and 3 retries are exhausted (circuit-breaker), (b) you detect potentially lost code (`git diff` shows far more deletions than additions with no new file created), or (c) Step 6 is explicitly requested. Any other pause is non-compliant with this skill.

---

## Step 2 — Establish a green baseline

Confirm the test suite is passing using the test command from the plan.

**If tests are already failing, stop and tell the user. Under no circumstances continue on a red baseline — not even if the failures appear pre-existing.** Pre-existing failures must be fixed or explicitly acknowledged in the plan before any refactoring starts. Do not offer to "proceed at your own risk" on a red baseline.

If a coverage tool is available, run it on the target file and compare against the coverage baseline in the plan.

**Coverage thresholds:**

| Coverage on files to touch | Recommendation |
|---|---|
| ≥ 70% lines | Safe — proceed |
| 40–69% lines | Caution — write characterisation tests first |
| < 40% lines | **Stop.** Strongly recommend writing tests first |
| 0% (no tests) | **Blocked.** Propose a minimal test harness, then restart |

If coverage is below 40%:
> "Coverage on the target file is X%. Refactoring without test coverage risks introducing silent regressions. Would you like me to suggest test cases based on the function signatures, or do you want to proceed at your own risk?"

Only continue past this point with **explicit user confirmation**.

**Large file warning**: if the target function spans more than 300 lines:
> "This function is X lines long. Small models may lose code when editing files of this size in a single pass. The plan must decompose each change to ≤ 50 lines. Verify the plan respects this before continuing."

---

## Step 3 — Set the restore point

Verify the working tree is clean:

```bash
git status            # must show: nothing to commit, working tree clean
git log --oneline -1  # note this commit hash — your restore point
```

If there are uncommitted changes, stop and ask the user to commit or stash them first.

Fill in the `Restore point` section of `.openlore/refactor-plan.md` with the current commit hash.

---

## Step 4 — Apply changes (mini-development loop)

For **each change** in the plan, execute the full mini-development cycle below.
Do not move to the next change until the current one is marked ✅.

### Before each change

Re-read `.openlore/refactor-plan.md` to confirm:
- Which change you are on (check for ✅ markers)
- Exactly what to extract, where to put it, and which call sites to update
- The test gate command for this specific change

### Editing tool rule

Always prefer a targeted edit tool over a full-file rewrite. Only use a full rewrite if the file is under 100 lines. If a change seems to require a full rewrite on a larger file, stop and split it into smaller targeted edits.

**Small model constraint**: each edit must touch a contiguous block of at most **50 lines**. If the planned change exceeds this, split it into sub-changes before proceeding — do not attempt an oversized edit.

### Mini-development cycle (execute for each change)

**Attempt counter**: reset to 1 at the start of each new change.

**1 — READ**
Re-read the source file around the lines to extract. Do not rely on memory or on earlier reads — the file may have changed from previous edits.

**2 — EDIT**
- Extract or move the identified block (≤ 50 lines)
- Place it in the target file and target class specified in the plan
- If the target file is new, create it with only the extracted code
- Update all call sites listed in the plan

**3 — DIFF**
Verify the edit before running tests:
```bash
git diff --stat   # only the expected files should appear
git diff          # scan deleted lines (−) and confirm each removal is
                  # intentional — moved, not silently dropped.
                  # If deleted lines >> added lines with no new file
                  # created, code was likely lost — abort immediately.
```

If the diff shows unexpected files or lost code (deletions >> additions, no new file):
```bash
git checkout HEAD -- <file>
```
Then re-examine the plan and retry from step 2 (counts as an attempt).

**4 — TEST**
Run the test gate from the plan entry. This is the exact command specified for this change.

```
Test result?
├─ GREEN → go to step 5 (mark done)
└─ RED   → git checkout HEAD -- <file>
           increment attempt counter
           if attempts < 3:
             diagnose the failure, adjust the edit, go back to step 2
           if attempts == 3:
             STOP — trigger circuit-breaker (see above)
```

**5 — MARK DONE**
Append `✅` to the change heading in `.openlore/refactor-plan.md`, then proceed to the next change.

---

## Step 5 — Verify improvement

Call the openlore MCP tool `analyze_codebase` with `{"directory": "$DIRECTORY", "force": true}`.

Then call the openlore MCP tool `get_refactor_report` with `{"directory": "$DIRECTORY"}`.

Check each acceptance criterion from the plan:
- Priority score dropped below the target
- Function is no longer in the top-5 list
- Full test suite passes

If not, investigate and iterate (add a new change to the plan if needed, respecting the ≤ 50 line constraint).

Run the full test suite one final time to confirm the refactored state is clean.

---

## Step 6 (optional — requires openlore generate to have been run)

> ⚠️ This step proposes irreversible changes (deletions, renames). Do not apply anything without explicit user confirmation at each sub-step.

### 6a — Dead code: orphan functions

Call the openlore MCP tool `get_mapping` with `{"directory": "$DIRECTORY", "orphansOnly": true}`.

Present the orphan list (kind `function` or `class` only). For each one, check:
- Is it exported and potentially consumed by external code?
- Is it re-exported from an index file?
- Was it simply missed by the LLM?

**Do not delete anything without the user explicitly approving each function.**

### 6b — Naming alignment: spec vocabulary vs actual names

Call the openlore MCP tool `get_mapping` with `{"directory": "$DIRECTORY"}`.

Build a table of mismatches and present it before touching any code:

| Current name | Proposed name | File | Confidence |
|---|---|---|---|

Only renames with `confidence: "llm"` should be proposed automatically. Flag `confidence: "heuristic"` entries for manual verification first.

**Wait for explicit user approval of the full rename table before applying any change. Apply renames one file at a time, run tests after each, and respect the ≤ 50-line edit constraint.**

---

## Absolute constraints

- Always re-read `.openlore/refactor-plan.md` before each change
- Never rewrite a file > 100 lines in a single operation
- Never accumulate broken state — restore immediately on any test failure
- Always verify the diff before running tests
- Never proceed to Step 6 without explicit user request
- Always flag potentially lost code (deleted lines >> added lines with no new file created)
- Never ask for confirmation between steps — only pause for circuit-breaker or lost-code signals
- Never continue on a red baseline, regardless of whether failures appear pre-existing
- Never attempt more than 3 retries on a single change without user input
- Each edit must touch ≤ 50 contiguous lines — split if needed, never skip this constraint
