<!-- BEGIN OPENLORE (managed — edits inside this block will be overwritten) -->
<!-- openlore-fingerprint: 25cdd746ebf39b56 -->
This project uses OpenLore for persistent architectural memory.

ALWAYS call `orient()` (via the openlore MCP server, or `npx openlore orient --json`)
before reading source files when starting a new task. This returns the relevant
functions, callers, spec sections, and insertion points for the task at hand —
one structural lookup instead of file-by-file rediscovery.

OpenLore prefixes tool responses with a brief, factual freshness note (the
Epistemic Lease) once your cached context has aged or the repo has moved since
your last `orient()`. It is informational — re-`orient()` if you are relying on
cached cross-module structure; otherwise carry on.

For the MCP setup, ensure `openlore mcp` is configured as an MCP server.
See https://github.com/clay-good/OpenLore for details.
<!-- END OPENLORE -->

<!-- openlore-decisions-instructions -->
## Architectural decisions

When making a significant design choice, call `record_decision` **before** writing the code.

Significant choices: data structure, library/dependency, API contract, auth strategy,
module boundary, database schema, caching approach, error handling pattern.

```
record_decision({
  title: "Use JWTs for stateless auth",
  rationale: "Avoids session store in infra",
  consequences: "Tokens can't be revoked early",
  affectedFiles: ["src/auth/middleware.ts"],
  supersedes: "a1b2c3d4"  // 8-char ID of prior decision being reversed
})
```

Decisions are consolidated in the background immediately after `record_decision` is called — the pre-commit gate reads the already-consolidated store and adds no LLM latency.

**Performance note**: if you skip `record_decision`, the gate detects unrecorded source changes at commit time and triggers a slow LLM extraction on the *next* commit (~10-30s). Calling `record_decision` proactively keeps every commit instant.

## When git commit is blocked by the decisions gate

If `git commit` fails and the output is JSON with `"gated": true`, do NOT retry silently.
Check the `reason` field and act accordingly:

**`reason: "verified"` — decisions await review:**
Present each decision to the user:
> "The commit is blocked — I found N architectural decision(s) to validate:
> 1. **[id]** Title — rationale
Do you approve? (yes/no)"
For each approval call `approve_decision`, for rejections call `reject_decision`.
Then run `openlore decisions --sync` and retry `git commit`.

**`reason: "approved_not_synced"` — decisions approved but not written to specs:**
Run `openlore decisions --sync` then retry `git commit`. Do not skip this step.

**`reason: "drafts_pending_consolidation"` — drafts were recorded but not yet consolidated:**
Present to the user:
> "N decision draft(s) were recorded but never consolidated. Run consolidation now? (~10-30s)"
If yes: run `openlore decisions --consolidate --gate` and handle the result.
If no: retry with `git commit --no-verify` to skip the gate.

**`reason: "no_decisions_recorded"` — source files staged but nothing recorded:**
Present to the user:
> "Source files are staged but no architectural decisions were recorded. Run fallback extraction to check for undocumented decisions? (~10-30s)"
If yes: run `openlore decisions --consolidate --gate` and handle the result.
If no: retry with `git commit --no-verify` to skip the gate.
<!-- end-openlore-decisions-instructions -->
