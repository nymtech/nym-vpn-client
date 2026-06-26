---
name: openlore-write-tests
description: Write real tests for a function or spec scenario — language-agnostic (TypeScript, Python, C++…). Reads the implementation and spec contract first, runs tests, fixes failures. No stubs, no placeholders.
license: MIT
compatibility: openlore MCP server
---

# openlore: Write Tests

## When to use this skill

Trigger this skill when the user wants to **write real tests** for a function, module, or spec
scenario, with phrasings like:
- "write tests for X"
- "add test coverage for Y"
- "what spec scenarios are untested?"
- "implement the test for this scenario"
- explicit command `/openlore-write-tests`

**The rule**: read the implementation and spec contract before writing a single assertion.
No `expect(true).toBe(true)`, no stubs, no placeholders.

**Prerequisite**: openlore analysis must exist (`openlore analyze` has been run).
If `orient` returns `"error": "no cache"` → run `openlore analyze` first, then retry.

---

## Step 1 — Identify target + detect framework

Ask the user:
1. **Target** — a function name, file path, spec scenario, or domain to test. If unsure, skip to Step 1b.
2. **`$PROJECT_ROOT`** — project root directory.

**Step 1b — Find untested scenarios (if no target given)**

Call the openlore MCP tool `get_test_coverage` with `{"directory": "$PROJECT_ROOT"}`.

Present the top 5 uncovered scenarios to the user, ranked by spec importance. Ask which to implement first.

**Step 1c — Detect test framework**

Scan the project root for framework config files:

| File found | Framework |
|---|---|
| `vitest.config.*`, `vitest.config.ts` | Vitest — runner: `npx vitest run <file>` |
| `jest.config.*` | Jest — runner: `npx jest <file>` |
| `pytest.ini`, `pyproject.toml` (with `[tool.pytest]`), `setup.cfg` | pytest — runner: `pytest <file> -v` |
| `CMakeLists.txt` with `enable_testing()`, `*.test.cpp` | CTest/GTest — runner: build + `ctest` |
| `go.mod` | Go test — runner: `go test ./...` |

Store as `$TEST_RUNNER`. If ambiguous, ask the user.

---

## Step 2 — Orient

Call the openlore MCP tool `orient` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "task": "write tests for $TARGET",
  "limit": 5
}
```

Extract:
- **`$TARGET_FILE`** — the file containing the function(s) to test
- **`$EXISTING_TEST_FILE`** — nearby test file, if any (e.g. `foo.test.ts`, `test_foo.py`, `foo_test.go`)
- **`$SPEC_DOMAIN`** — the spec domain associated with the target

---

## Step 3 — Read implementation + spec contract

**This step is mandatory. Do not write any test before completing it.**

**3a — Read the function body**

Call the openlore MCP tool `get_function_body` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "symbol": "$TARGET_FUNCTION",
  "filePath": "$TARGET_FILE"
}
```

Identify:
- What the function takes as input, what it returns
- What external dependencies it calls (filesystem, network, DB, LLM, subprocess)
- What invariants are visible (guards, throws, early returns)

**3b — Find the spec contract (if specs exist)**

Call the openlore MCP tool `search_specs` with:
```json
{
  "directory": "$PROJECT_ROOT",
  "query": "$TARGET — expected behaviour",
  "limit": 5
}
```

For each matching spec scenario, note:
- The **GIVEN / WHEN / THEN** clauses — these become the test body
- The scenario name — this becomes the `it()` / `def test_` / `TEST()` description

If no specs exist, infer the contract from the function signature, docstring, and call sites.
Document the inferred contract explicitly before writing any test.

**3c — Absorb local test conventions**

If `$EXISTING_TEST_FILE` exists, read it. Extract:
- Mock setup pattern (`vi.mock`, `unittest.mock.patch`, `gmock`, etc.)
- Fixture or factory helpers
- Import path style (relative vs absolute)
- Describe/class/suite structure

If no test file exists nearby, find the closest test file in the project tree and read that instead.

---

## Step 4 — Write tests

Write (or append to) `$EXISTING_TEST_FILE` (or create `<name>.test.ts` / `test_<name>.py` / `<name>_test.go` next to the source file).

### Rules — enforced without exception

1. **No placeholder assertions** — `expect(true).toBe(true)`, `assert True`, `EXPECT_TRUE(true)`,
   `self.assertTrue(True)` are forbidden. Every assertion must test real return values or side effects.

2. **One test = one scenario** — each `it()` / `def test_` / `TEST()` maps to one GIVEN/WHEN/THEN
   clause. Use the spec scenario name (or a descriptive contract statement) as the test description.

3. **Annotation tag (mandatory)** — place a coverage tag on the line immediately above each
   `describe` / class / suite block so `openlore test --coverage` can track it:
   - TypeScript/JS: `// openlore: {"domain":"$DOMAIN","requirement":"$REQ","scenario":"$SCENARIO","specFile":"openspec/specs/$DOMAIN/spec.md"}`
   - Python: `# openlore: {"domain":"$DOMAIN","requirement":"$REQ","scenario":"$SCENARIO"}`
   - C++/Go: `// openlore: {"domain":"$DOMAIN","requirement":"$REQ","scenario":"$SCENARIO"}`

   If no spec scenario exists (contract inferred), omit the tag.

4. **Mock only system boundaries** — mock filesystem, network, LLM API, DB connections, and
   external processes. Do not mock the function under test, its pure helpers, or in-memory logic.

5. **One suite per function** — `describe` / class / suite named after the function.
   Use nested blocks for distinct concerns (happy path, error path, edge cases).

6. **At least one edge case** — empty input, null/None/nullptr, maximum value, or an error path
   must be included for every function tested.

7. **Small model constraint** — if the test file exceeds 200 lines, split into multiple files
   grouped by concern. Each file must be independently runnable.

### Structure reference (adapt to the detected framework)

```typescript
// TypeScript / Vitest
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { $TARGET_FUNCTION } from '../$TARGET_FILE';

vi.mock('../$DEPENDENCY', () => ({ ... }));

// openlore: {"domain":"$DOMAIN","requirement":"$REQUIREMENT","scenario":"$SCENARIO","specFile":"openspec/specs/$DOMAIN/spec.md"}
describe('$TARGET_FUNCTION', () => {
  beforeEach(() => { vi.resetAllMocks(); });

  describe('$SCENARIO_NAME', () => {
    it('should $EXPECTED_BEHAVIOUR when $CONDITION', () => {
      // GIVEN
      // WHEN
      const result = $TARGET_FUNCTION($INPUT);
      // THEN
      expect(result).toEqual($EXPECTED);
    });
  });
});
```

```python
# Python / pytest
import pytest
from unittest.mock import patch, MagicMock
from $MODULE import $TARGET_FUNCTION

# openlore: {"domain":"$DOMAIN","requirement":"$REQUIREMENT","scenario":"$SCENARIO"}
class Test$TargetFunction:
    def test_$scenario_name_when_$condition(self):
        # GIVEN / WHEN / THEN
        result = $target_function($input)
        assert result == $expected
```

---

## Step 5 — Run and fix

Run the test file with `$TEST_RUNNER`. Iterate until all tests pass.

| Outcome | Action |
|---|---|
| All green | Proceed to Step 6 |
| Failure in new test | Diagnose: is the assertion wrong, or is there a real bug? Fix the assertion if the expectation was incorrect. If a real bug is revealed, do not weaken the assertion — report the bug instead. |
| Failure in pre-existing test | Stop. Fix the regression before adding more tests. |
| Test can't compile / import | Fix the import path, mock setup, or dependency injection before retrying. |

Do not weaken assertions to make tests pass. A test that masks a bug is worse than no test.

---

## Step 6 — Coverage check

Call the openlore MCP tool `get_test_coverage` with `{"directory": "$PROJECT_ROOT"}`.

Report:
- Which spec scenarios are now covered (new)
- Which scenarios remain uncovered in `$SPEC_DOMAIN`
- Whether any hub functions in `$SPEC_DOMAIN` are still untested (high-value next targets)

---

## Absolute constraints

- Never write `expect(true).toBe(true)`, `assert True`, or equivalent placeholder assertions
- Never skip Step 3 — the implementation read and spec contract are the test source of truth
- Never mock the function under test itself
- Never weaken an assertion to make a test pass — fix the implementation or the expectation
- If `get_test_coverage` shows the scenario is already covered, report it and stop
- Do not refactor the implementation as part of this skill — open a separate task
