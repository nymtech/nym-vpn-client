## Context

`src/ui/Slider.tsx` wraps `@base-ui-components/react` `Slider.Root`. It keeps an
`internalValue` state, synced from the `value` prop and updated on `onValueChange`. The
commit handler is wired as:

```tsx
onValueCommitted={() => onValueCommitted?.(internalValue)}
```

base-ui's `onValueCommitted` provides the committed value as a callback argument, but this
code discards it and reads the closed-over `internalValue`. React state updates are not
synchronous: on keyboard/click commits, base-ui fires `onValueChange` (which calls
`setInternalValue`) and `onValueCommitted` in the same tick, so the commit closure still sees
the pre-interaction `internalValue`. `onValueChange` is already wired correctly
(`(val) => { setInternalValue(val); onChange?.(val); }`) — only the commit path is wrong.

This was found while adding tests: the UiScaler and mixnet-tuning card tests had to assert the
displayed value instead of the persisted value to stay green, which hid the defect.

## Goals / Non-Goals

**Goals:**
- Slider forwards the actual committed value to `onValueCommitted` in all commit paths.
- Update the tests that worked around the bug to assert correct behavior, plus a regression
  test for the synchronous (keyboard) commit path.

**Non-Goals:**
- No change to `Slider`'s public props/API.
- No refactor of `internalValue`/controlled-value handling beyond the commit wiring.
- No behavior change to `onValueChange` (already correct).

## Decisions

### Forward the callback argument instead of the closed-over state
Change the wiring to `onValueCommitted={(val) => onValueCommitted?.(val)}`, mirroring the
existing `onValueChange` handler. This is the minimal, correct fix: the argument is the fresh
committed value from base-ui and is never stale.

*Alternatives considered:*
- **`useRef` mirror of the latest value** read in the commit handler — works but adds state
  plumbing to dodge a problem the callback argument already solves. Rejected as unnecessary.
- **`flushSync` on the change** to force `internalValue` current before commit — heavier,
  perf-affecting, and still indirect. Rejected.

### Type of the committed value
base-ui's slider value is `number | readonly number[]`; every app consumer is a single-thumb
slider and `internalValue` is typed `number`, matching the existing `onValueChange` handler.
Forward the value the same way `onValueChange` does so the types stay consistent (single
`number`); no new casts beyond what `onValueChange` already relies on.

## Risks / Trade-offs

- **A consumer relied on the stale/echoed prop value** → None found; all consumers pass an
  `onValueCommitted` that expects the committed value (UiScaler, mixnet-tuning). The fix makes
  them correct.
- **Tests asserting the old (buggy) behavior break** → Expected and intended; those tests are
  updated in this change to assert the correct committed value.

## Migration Plan

Single-file source fix plus test updates; no data or API migration. Verify with
`npm run test`, `npm run tscheck`, `npm run lint`, and `npm run fmt:check`.
