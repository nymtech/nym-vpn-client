# Example: from task to first edit

**User:** "Add a rate limiter to the API client."

The agent picks up this skill and, before opening any source files, calls:

```sh
bash scripts/orient.sh "add a rate limiter to the API client"
```

The output (see `example-orient-output.json` for the actual shape) returns:

- `relevantFunctions[]` — pointing at `createLLMService` in `src/core/services/llm-service.ts` (the factory every command flows through) and `fetchWithRetry` in `src/core/services/chat-agent.ts` (the HTTP boundary).
- `callPaths[]` — every CLI command that calls into the LLM service (`doctor`, `verify`, `drift`, `generate`), so the agent knows the blast radius of a behavior change.
- `specDomains[]` — surfacing the `services` and `chat` spec domains and the requirements about how request handling is documented.
- `insertionPoints[]` — ranking the factory as the top candidate (wrap once, every caller benefits), with `fetchWithRetry` as a fallback if the rate limit needs token-bucket-per-provider awareness.

The agent then proceeds in this order, *without* opening anything else first:

1. Reads the two `spec_sections` to understand the documented contract.
2. Reads the callers list to confirm which call paths must keep working.
3. Opens `llm-service.ts` at the line from the top insertion point — not the top of the file.

Total tokens consumed before the first edit: ~3k for the `orient` output + targeted reads. Without the skill, the same task typically costs 30k–50k tokens of file-by-file exploration before the agent finds the right place to edit.

The Epistemic Lease keeps watch from this point on: if a tool response later carries a "stale" prefix (because some unrelated edit moved the graph forward), the agent re-orients before continuing.
