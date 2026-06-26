---
name: openlore-brainstorm
description: Transform a feature idea into an annotated story. Detects greenfield vs brownfield automatically — uses Domain Sketch for greenfield (no existing code), Constrained Option Tree for brownfield (existing codebase with openlore analysis).
license: MIT
compatibility: openlore MCP server
---

# openlore: Brainstorm

## When to use this skill

Trigger this skill when the user wants to **explore or design a new feature** before
writing any code, with phrasings like:
- "I want to add feature X"
- "how should I approach this?"
- "let's brainstorm this story"
- explicit command `/openlore-brainstorm`

---

## Step 1 — Detect mode

Capture `$PROJECT_ROOT`, `$FEATURE_DESCRIPTION`, and
`$FEATURE_SLUG` (kebab-case, ≤ 5 words, e.g. `payment-retry-flow`).

Then probe the structural index by calling the openlore MCP tool `orient` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "task": "$FEATURE_DESCRIPTION",
  "limit": 7
}
```

Set `$MODE` based on the result:

| Result | `$MODE` | Meaning |
|---|---|---|
| Returns functions with score > 0 | `brownfield` | Existing indexed codebase |
| Returns `"error": "no cache"` or 0 functions | `greenfield` | No structural index yet |

Inform the user:
- Brownfield: "I found existing code related to this feature. Using structural analysis to guide design."
- Greenfield: "No structural index found. Using Domain Sketch — structural analysis will be available after `openlore analyze` is run on the first implementation."

**Load project antipatterns (both modes):**

If `.claude/antipatterns.md` exists, read it and store as `$ANTIPATTERNS`.
These will be used as a failure mode source in Step 5. If absent, `$ANTIPATTERNS = none`.

---

## Steps 2–4 — Structural analysis (brownfield only)

*Skip to Step 5 if `$MODE = greenfield`.*

### Step 2 — Architecture overview

Call the openlore MCP tool `get_architecture_overview` with `{"directory": "$PROJECT_ROOT"}`.

Note hub functions and cross-domain dependencies in `$DOMAINS_AFFECTED`.

### Step 3 — Generate change proposal

Call the openlore MCP tool `generate_change_proposal` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "description": "$FEATURE_DESCRIPTION",
  "slug": "$FEATURE_SLUG"
}
```

This chains `orient` + `search_specs` + `analyze_impact` and writes
`openspec/changes/$FEATURE_SLUG/proposal.md`.

Extract:
- **`$MAX_RISK_SCORE`** — overall risk level
- **`$REQUIREMENTS_TOUCHED`** — existing requirements this feature overlaps
- **`$BLOCKING_REFACTORS`** — functions with risk ≥ 70

### Step 4 — Risk gate

| Score | Action |
|---|---|
| 🟢 < 40 | Proceed to Step 5 |
| 🟡 40–69 | Proceed, flag impacted callers to protect during design |
| 🔴 ≥ 70 | Stop — propose a blocking refactor story before continuing |

If blocked:
> "This feature touches `$BLOCKING_FUNCTION` (risk score: $SCORE).
> A refactor story must be completed first. I can create it now if you'd like."

Do not continue until the user accepts the refactor story or explicitly overrides.

---

## Step 5 — Brainstorming

### If `$MODE = brownfield` — Constrained Option Tree

**5a — Establish the constraint space**

Derive from Steps 2–4 and present before generating options:

```
Hard constraints (non-negotiable):
  - Functions with riskScore ≥ 70: $BLOCKED_FUNCTIONS
  - Requirements that must be preserved: $REQUIREMENTS_TOUCHED
  - Domain boundaries that must not be crossed: $DOMAIN_BOUNDARIES

Soft constraints (preferred):
  - Existing insertion points: $INSERTION_POINTS
  - Patterns already used in $DOMAINS_AFFECTED
```

Ask: "Are there additional constraints before we explore approaches?"

**5b — Generate 2–3 options**

Each option must respect hard constraints. Name them clearly
(e.g. Option A — Extend existing, Option B — New service, Option C — Facade).

If `$ANTIPATTERNS ≠ none`, cross-check each option against the list.
Flag any option that would reproduce a known failure pattern with ⚠ and the relevant AP-NNN.

| | Option A | Option B | Option C |
|---|---|---|---|
| Insertion point | | | |
| Domains touched | | | |
| Risk score impact | | | |
| Requirements affected | | | |
| Estimated scope (files) | | | |
| Trade-off | | | |

**5c — Recommend**

State one recommendation with a single structural reason:

> "Recommend Option B — inserts at `$SAFE_FUNCTION` (risk 18), avoids `$HUB`
> (fan-in 14) entirely. Option A is valid but adds unnecessary blast radius."

**5d — Confirm**

Do not proceed to Step 5e until the user chooses. If they want a hybrid,
produce a revised option table.

**5e — Adversarial challenge**

Before writing the story, switch roles: challenge the chosen option as a skeptic.

State exactly 2 failure modes grounded in the structural data and, if `$ANTIPATTERNS ≠ none`,
any applicable antipatterns from `.claude/antipatterns.md`:

> "Devil's advocate on Option B:
> 1. `$CALLER_A` calls `$INSERTION_POINT` with `$EDGE_CASE` — this path is not covered
>    by the proposed approach and could silently break.
> 2. The proposal scores risk at $SCORE but `$HUB` (fan-in $N) is one hop away —
>    a regression there would not be caught until `check_spec_drift`."

Then ask the user: "Do these failure modes change your choice, or should we add
mitigations to the story constraints?"

Only proceed to Step 6 once the user has acknowledged the failure modes.
Mitigations go into `## Technical Constraints` in the story.

> Note: this IS a gate (waits for user input) because brainstorm is a design phase
> where changing course is cheap. In contrast, `openlore-implement-story` Step 4b
> is a mandatory self-check that does NOT gate — because by implementation time
> the design decision is already made.

Ask: "What is explicitly out of scope for this story?" List the answers as `$WONT_DO`.

---

### If `$MODE = greenfield` — Domain Sketch

No existing structure to constrain — the method builds the structure from scratch.

**5a — Entities**

Ask the user: "What are the core data objects this feature creates or transforms?"

List them as nouns with a one-line definition each:
```
$ENTITY_1 — definition
$ENTITY_2 — definition
```

**5b — Operations**

For each entity, identify the operations the feature needs (create, read,
transform, delete, emit, receive…):

```
$ENTITY_1: $OP_1, $OP_2
$ENTITY_2: $OP_1
```

Identify which operations cross a system boundary (external API, database,
event bus, CLI…) — these are the riskiest integration points.

**5c — Boundaries**

Define where the feature sits relative to the system:

```
Entry point:   $HOW_IT_IS_TRIGGERED (HTTP request / CLI command / event / cron)
Data in:       $INPUT_FORMAT
Data out:      $OUTPUT_FORMAT or side-effects
External deps: $THIRD_PARTY_SERVICES or "none"
```

**5d — Architecture decisions**

State 2–3 decisions that must be made before coding. For each, list the options
and a recommendation:

| Decision | Options | Recommendation |
|---|---|---|
| e.g. Storage | in-memory / DB / file | DB — survives restarts |
| e.g. Coupling | new module / extend existing | new module — clear boundary |

Ask the user to confirm or override each decision before proceeding to Step 6.

Ask: "What is explicitly out of scope for this story?" List the answers as `$WONT_DO`.

---

## Step 6 — Write the story

Produce a story file at `$STORIES_DIR/$FEATURE_SLUG.md`.

If a story template exists at `$PROJECT_ROOT/_bmad/openlore/templates/story.md`
or `$PROJECT_ROOT/examples/bmad/templates/story.md`, use it. Otherwise:

**Brownfield template:**

```markdown
# $STORY_TITLE

## Goal

$FEATURE_DESCRIPTION

## Acceptance Criteria

Each AC must be verifiable by a test. State the observable outcome, not the intent.
✗ "Should handle errors correctly" — ✓ "Returns HTTP 400 with `{error: 'X'}` when Y is absent"

- [ ] $AC_1
- [ ] $AC_2

## Won't Do

- $WONT_DO_1
- $WONT_DO_2

## Risk Context

<!-- Filled by annotate_story in Step 7 -->

## Technical Constraints

$BLOCKING_REFACTORS and caller protection notes from the proposal.

## Notes

- Chosen approach: $CHOSEN_OPTION — $TRADE_OFF
- Domains affected: $DOMAINS_AFFECTED
- Requirements touched: $REQUIREMENTS_TOUCHED
- Max risk score: $MAX_RISK_SCORE
```

**Greenfield template:**

```markdown
# $STORY_TITLE

## Goal

$FEATURE_DESCRIPTION

## Acceptance Criteria

Each AC must be verifiable by a test. State the observable outcome, not the intent.
✗ "Should handle errors correctly" — ✓ "Returns HTTP 400 with `{error: 'X'}` when Y is absent"

- [ ] $AC_1
- [ ] $AC_2

## Won't Do

- $WONT_DO_1
- $WONT_DO_2

## Domain Sketch

### Entities
$ENTITIES

### Operations
$OPERATIONS

### Boundaries
$BOUNDARIES

## Architecture Decisions

$DECISIONS_TABLE

## Notes

- First implementation — run `openlore analyze && openlore generate` after
  to enable structural analysis for future stories.
```

---

## Step 7 — Annotate the story

**Brownfield only.** Skip if `$MODE = greenfield`.

Call the openlore MCP tool `annotate_story` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "storyFilePath": "$STORY_FILE_PATH",
  "description": "$STORY_TITLE"
}
```

Patches `## Risk Context` directly. Story is now ready for `openlore-implement-story`.

**Greenfield:** confirm to the user:
> "Story written to `$STORY_FILE_PATH`. No risk context yet — run
> `openlore analyze` after the first implementation sprint to enable
> structural analysis for future stories."

---

## Absolute constraints

- Do not ask design questions before Step 5 (both modes)
- Brownfield: do not proceed past Step 4 if `$MAX_RISK_SCORE ≥ 70` without acknowledgement
- Brownfield: do not fill `## Risk Context` manually — always use `annotate_story`
- Greenfield: do not call `annotate_story` — there is nothing to annotate yet
- Do not propose implementation steps — this skill ends at story creation
- Every AC must be verifiable by a test — reject vague ACs ("should work", "handles errors") and rewrite them before proceeding
- `## Won't Do` is mandatory in the story — at minimum one item
- Brownfield: `generate_change_proposal` creates `openspec/changes/$FEATURE_SLUG/proposal.md`
  on disk. Inform the user at session end:
  "A proposal file was created at `openspec/changes/$FEATURE_SLUG/proposal.md`.
  Delete it if this idea is not pursued."
