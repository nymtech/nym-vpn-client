## 1. Reproduce (failing test first)

- [x] 1.1 In `src/ui/Slider.test.tsx`, add a test that drives a synchronous (keyboard) commit and asserts `onValueCommitted` receives the NEW value — confirmed it FAILED against current code (got 30/40 instead of 40/50, proving the stale-value bug)

## 2. Fix

- [x] 2.1 In `src/ui/Slider.tsx`, change `onValueCommitted={() => onValueCommitted?.(internalValue)}` to forward base-ui's callback argument: `onValueCommitted={(val) => onValueCommitted?.(val)}`
- [x] 2.2 Confirm the new regression test now PASSES

## 3. Update worked-around tests to assert correct behavior

- [x] 3.1 `src/screens/settings/appearance/display/UiScaler.test.tsx` — added a keyboard-commit test asserting the committed size sets `document.documentElement.style.fontSize` and persists via `db_set` (value 15, not the stale 14)
- [x] 3.2 mixnet-tuning cards: `MixingDelayCard` uses `onChange` (not affected — no change). `ContinuousTrafficCard` uses `onValueCommitted` — added a keyboard-commit test asserting the value advances (a stale commit would snap it back)
- [x] 3.3 `src/ui/Slider.test.tsx` covers keyboard commit forwarding the correct value (single + successive steps); base-ui drag isn't simulable in jsdom, keyboard is the synchronous commit path that exposed the bug

## 4. Verify all gates

- [x] 4.1 `npm run test` — 726 passed
- [x] 4.2 `npm run tscheck` — 0 errors
- [x] 4.3 `npm run lint` — 0 errors
- [x] 4.4 `npm run fmt:check` — clean for touched files
