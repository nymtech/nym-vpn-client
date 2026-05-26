# Auto / Fast / Mixnet Mode Toggle — Feature Reference

Branch: `nym-1390-1click-enhancements`. Companion to NYM-1385 ("3 state toggle — Auto / Fast / Mixnet, enhancement to NYM-461").

This file is the load-anywhere brief for the new home-screen mode toggle and the
gateway-selection flow that ships with it. It supersedes the
[`NewBottomComponent.fold-logic.md`](./NewBottomComponent.fold-logic.md)
reference — the **chevron / fold state machine described there is dead code**
(left in place behind comments for the moment, see "Preserved chevron
scaffolding" below).

---

## TL;DR — the model

A single 3-state pill (`ModeToggle`) at the top of the home card drives both
`GatewaySelectionAlgorithm` and `VpnMode`. Both entry and exit `NodeRow`s are
always rendered.

| Pill       | `gatewaySelectionAlgorithm`           | `vpnMode`  | Who picks what                                 |
| ---------- | ------------------------------------- | ---------- | ---------------------------------------------- |
| **Auto**   | `'auto'` or `'autoEntryExplicitExit'` | unchanged  | Daemon picks both, or daemon picks entry only. |
| **Fast**   | `'explicit'`                          | `'wg'`     | User picks both.                               |
| **Mixnet** | `'explicit'`                          | `'mixnet'` | User picks both.                               |

The visual pill is **derived from the store**, not a local `useState`. It moves
in response to _any_ state change so the toggle and the rows stay in sync.

**The entry row is unclickable when daemon-picked.** When `daemonPicked` is
true _and_ `type === 'entry'` (so: entry in `'auto'`, entry in
`'autoEntryExplicitExit'`) the row has no click handler and no hover border.
While `connecting`/`connected` the row's content is also dimmed
(`opacity-60`); the `info` `ButtonIconNew` stays at full opacity and remains
clickable so the user can still open server details for the daemon's pick.

The **exit row stays interactive in every algo.** Picking an exit in `'auto'`
is exactly what flips us into `'autoEntryExplicitExit'` (the daemon keeps the
entry, the user now owns the exit). The `Best server` quick-pick in the exit
list is the escape hatch back to `'auto'`.

Practical consequence: the entry list is reachable only from `'explicit'`
(Fast or Mixnet pill), since that is the only algo where the entry row is
clickable. The mirror flip "entry-pick → explicit" that used to live in
`Node.tsx` / `NodeDetails.tsx` is therefore dead code and has been removed —
only the exit-pick `'auto' → 'autoEntryExplicitExit'` flip remains.

### Exit pick persists across mode switches

`DefaultNode` is now `'random'` (was `{country: 'CH'}`), which makes
`exitNode === 'random'` the **"no user pick"** sentinel. Two places lean on
that:

1. **`ModeToggle` Auto pill** — instead of always setting `algo = 'auto'`,
   `handleSelect('auto')` looks at `exitNode`:
   - `exitNode === 'random'` → `algo = 'auto'` (daemon picks both).
   - `exitNode` is a country/region/gateway → `algo = 'autoEntryExplicitExit'`
     (preserve the user's exit). So going `auto → fast → auto` no longer
     resets the exit to "Best server".

2. **`Node.tsx handleBestServer`** — Best server in the exit list now also
   calls `set_node('random', 'exit')` before flipping `algo` to `'auto'`.
   Without the reset, ModeToggle would later see a stale exit selection and
   bounce us into `'autoEntryExplicitExit'` instead of staying in `'auto'`.

There is **no `Best server` quick-pick in the entry list.** The "Best server"
concept exists only as the daemon-picked label shown on the entry row in
auto-style algos.

```ts
selected =
  algo === 'auto' || algo === 'autoEntryExplicitExit'
    ? 'auto'
    : vpnMode === 'wg'
      ? 'fast'
      : 'mixnet';
```

---

## Files & responsibilities

| File                                                      | Role                                                                                                                                                                                                                                               |
| --------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/screens/home/ModeToggle.tsx`                         | Renders the 3-pill segmented control; reads algo + vpnMode from store; dispatches `set_gateway_selection_algorithm` + `set_vpn_mode` on selection.                                                                                                 |
| `src/screens/home/NewBottomComponent.tsx`                 | The home card. Now renders `<ModeToggle />` above the card and unconditionally renders both `<NodeRow type="exit" />` + `<NodeRow type="entry" />`. The chevron / `foldState` machinery is preserved but commented out.                            |
| `src/screens/home/NodeRow.tsx`                            | Per-hop row: `Best server` placeholder, country/gateway display, `Skeleton` loading state, navigation to node list. Handles "daemon-picked" semantics so a stored selection from a previous mode never leaks into a row the daemon currently owns. |
| `src/screens/node/NodeLocation.tsx`                       | Tabbed wrapper around the entry/exit lists. **Both tabs are now always shown**; initial active tab comes from `location.state.tab`.                                                                                                                |
| `src/screens/node/Node.tsx`                               | The list itself. Renders two quick-pick rows (Random, Best server) above the country list, and owns the post-selection algorithm/vpnMode flips described below.                                                                                    |
| `src/screens/node/details/NodeDetails.tsx`                | Mirror of `Node.tsx`'s selection commit: same exit→`autoEntryExplicitExit` and entry→`explicit`+`wg` flips fire from here too.                                                                                                                     |
| `src/hooks/useNodeListData.ts`                            | Maps the cached gateway lists into UI nodes. Now suppresses the `isSelected` highlight for hops the daemon picks (`effectiveEntry` / `effectiveExit` are forced to `'random'` so `isSelectedNodeType` returns `false`).                            |
| `src/assets/icons/gateway-mode/{auto,fast,anonymous}.svg` | Pill icons. All three use `fill="currentColor"` so Tailwind `text-*` classes drive the icon color. (`mixnet.svg` was renamed to `anonymous.svg` to match the broken import in `index.ts`.)                                                         |
| `src/i18n/en/home.json`                                   | Existing `toggle-vpn-mode` namespace still used for `toggle-vpn-mode.error` + `gateway-selection-algorithm.error` toasts. The pill labels (Auto / Fast / Mixnet) are currently **hardcoded** — i18n keys to be added later.                        |

---

## The ModeToggle (`src/screens/home/ModeToggle.tsx`)

Three buttons inside a rounded pill. The active slot has a `motion.div` with
`layoutId="mode-toggle-pill"` so when the selection changes the highlight
slides between buttons (300 ms `easeOut`).

### `handleSelect(mode)`

```ts
if (mode === selected) return; // clicking the active pill is a no-op

const vpnModeToSet = mode === 'auto' || mode === 'fast' ? 'wg' : 'mixnet';

// Auto: preserve any explicit exit pick by going to autoEntryExplicitExit.
// Fast / Mixnet: always 'explicit'.
const algorithmToSet =
  mode === 'auto'
    ? exitNode === 'random'
      ? 'auto'
      : 'autoEntryExplicitExit'
    : 'explicit';

await applyVpnMode(vpnModeToSet);
await applyAlgorithm(algorithmToSet);
```

`applyAlgorithm` and `applyVpnMode` each:

1. Skip if the value already matches (`if (algorithm === algo) return;`).
2. `invoke(...)` the Tauri command.
3. Dispatch the Zustand reducer on success.
4. On failure: `console.error` + toast (`gateway-selection-algorithm.error` /
   `toggle-vpn-mode.error`).

`applyVpnMode` also kicks `fetchGateways` so the relevant gateway list
(`'wg'` or `'mx-entry'` + `'mx-exit'`) starts loading immediately.

> Today the pill switches `vpnMode` directly. Per NYM-1385 the library is
> meant to auto-reconnect on mode change — we are not invoking `connect` /
> `disconnect` here on purpose.

---

## NodeRow (`src/screens/home/NodeRow.tsx`)

The same component renders both hops. The most important concept introduced on
this branch is:

```ts
const daemonPicked =
  algo === 'auto' || (algo === 'autoEntryExplicitExit' && type === 'entry');
```

This boolean drives **every read of `userSelectedNode` and every textual
fallback**. When a hop is daemon-picked the stored `entryNode` / `exitNode` is
treated as stale UI state (it may still contain a value from a previous
mode) — the row is derived purely from the daemon-reported gateway.

### Gateway resolution

```ts
const gateway = useMemo(() => {
  const gw = type === 'entry'
    ? tunnel?.entryGwId || connectingState?.entryGwId
    : tunnel?.exitGwId || connectingState?.exitGwId;

  if (daemonPicked)                    return gw ? lookupGw(gw, type) : null;
  if (isGateway(userSelectedNode))     return lookupGw(userSelectedNode.gateway.id, type);
  return gw ? lookupGw(gw, type) : null;
}, [..., wg, mxEntry, mxExit]);
```

**Critical**: the `wg` / `mxEntry` / `mxExit` arrays are in the dependency
array even though `lookupGw` reads them via the store. `lookupGw` has a stable
identity (it lives on the slice and never changes), so without the list refs
the memo would never re-run after the lists arrive from `fetchGateways` and
the row would be stuck showing the raw gateway ID. _Do not drop these deps._

### `nodeDetails`

- `daemonPicked` → `getGatewayInfo(gateway?.id ?? '', gateway)` — same path
  the legacy `algo === 'auto'` case used.
- `userSelectedNode === 'random'` AND state is `connecting`/`connected` AND
  `gateway` is non-null → also `getGatewayInfo(gateway.id, gateway)`. This is
  how Random shows the actual daemon-picked server once we're connected
  instead of the generic "Random server" placeholder.
- Otherwise → `nodeData(userSelectedNode, gateway)` (the existing
  country/region/gateway routing).

### `textLabel`

- `daemonPicked`: `nodeDetails.ip ?? 'Best server for my location'`.
- `algo === 'autoEntryExplicitExit'` (exit only after `daemonPicked` filter):
  `state === 'connected' ? nodeDetails.ip : nodeDetails.name`.
- `algo === 'explicit'`: `state === 'connected'
? (gateway?.name ?? nodeDetails.name) : nodeDetails.name`.

### Loading state (Skeleton)

Mode switches change which gateway list is authoritative; while the new list
is fetching, `lookupGw` returns `null` and the legacy fallback would render
the raw gateway ID. The component now tracks:

```ts
const listLoading =
  (vpnMode === 'wg'     && wgLoading) ||
  (vpnMode === 'mixnet' && type === 'entry' && mxEntryLoading) ||
  (vpnMode === 'mixnet' && type === 'exit'  && mxExitLoading);

const hasGatewayIdToResolve =
  Boolean(tunnel?.[Gw]Id || connectingState?.[Gw]Id) ||
  (!daemonPicked && isGateway(userSelectedNode));

const showLoading = listLoading && !gateway && hasGatewayIdToResolve;
```

When `showLoading` is true the text slot renders `<Skeleton className="h-5
w-40" />` and `descriptionLabel` short-circuits to `null` so only one shimmer
line shows. Once the list lands and `gateway` resolves, the row flips back to
its normal display.

### `handleClick`

Only runs when the row is clickable (not `daemonPicked`):
`reset(type)` → focus/expand the user's stored selection →
`navigate(routes.nodeLocation, { state: { tab: type } })`. No mode/algo flips
happen here. On daemon-picked rows `handleClick` is not wired up — see
`rowDisabled` above.

### FlagIcon visibility

```ts
nodeDetails.countryCode &&
  (state === 'connected' || state === 'connecting' || !daemonPicked);
```

For daemon-picked hops when idle, `gateway` is `null` so `nodeDetails.countryCode`
is `undefined` anyway — but the condition guards the connecting/connected
case explicitly.

---

## Node list (`src/screens/node/Node.tsx`)

### Quick-pick rows

Two buttons render above the country list (inside the scrollable area):

| Row             | Shown                                                                                                 | Click                                                                                                                                                   | Highlighted when                                                                                                                                            |
| --------------- | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Random**      | Always (both entry and exit, in every algo)                                                           | Calls `handleSelect({ nodeType: 'random', isSelected: false })` → goes through the same `set_node` path as picking a real gateway → algo/vpnMode flips. | `storedNode === 'random'` AND the user actually owns the hop: entry → `algo === 'explicit'`; exit → `algo !== 'auto'`. (Mirrors `useNodeListData`'s logic.) |
| **Best server** | Exit list, only when `algo` is `'auto'` or `'autoEntryExplicitExit'` (i.e., the Auto pill is active). | Calls `set_gateway_selection_algorithm` with `'auto'` if needed (no `set_node`), then navigates back.                                                   | `algo === 'auto'`.                                                                                                                                          |

Selection styling reuses the country-row idiom: `border-2 border-primary-active`.

### Selection commit — algo flip

`handleSelect(selected)`:

1. `set_node` + dispatch `set-node`.
2. If `node === 'exit'` and current `algo === 'auto'` →
   `set_gateway_selection_algorithm: 'autoEntryExplicitExit'`.

The mirror flip `non-explicit → 'explicit'` on entry-pick that used to live
here is gone: the entry row is unclickable in both auto algos, so the entry
list is only reachable from `'explicit'`, where that flip would be a no-op.

`NodeDetails.handleSelect` carries the exact same exit-only flip. **Keep them
in sync.**

The early "already-selected" return:

```ts
if (
  isGateway(selectedNode) &&
  (selected.isSelected === 'exit' || selected.isSelected === 'entry')
) {
  return;
}
```

…only fires for `Gateway`-typed selections. `Random` / `Best server` /
country / region selections never trip it, and `useNodeListData` now strips
the highlight on daemon-picked hops so the user can re-pick the same node in
auto mode.

---

## `useNodeListData` — effective entry/exit

```ts
const effectiveEntry = algo === 'explicit' ? entryNode : 'random';
const effectiveExit = algo === 'auto' ? 'random' : exitNode;
```

These two values are passed to `buildNodeList` in place of the raw store
selections. `isSelectedNodeType` treats `'random'` as "no country/gateway
matches", so the country/region/gateway rows render unhighlighted and the
existing `Node.handleSelect` early-return no longer blocks re-picking a node
that was selected in a different mode.

The quick-pick rows compute their own `randomActive` / `bestServerActive`
booleans directly from algo + storedNode rather than going through this
hook — they need to know "is _this exact storedNode_ random" not "should the
list be highlighted".

---

## `NodeLocation` — both tabs always

The previous gate `showEntryTab = algo === 'explicit'` is gone. Both tabs are
rendered unconditionally; the initial active tab is `location.state.tab ??
'exit'`. This is required because the entry row is clickable in every algo
now (the flip is deferred until selection).

---

## Preserved chevron scaffolding

`NewBottomComponent.tsx` still contains the following symbols, intentionally
left in for a possible later iteration:

- `import { useEffect } from 'react'`
- `import { AnimatePresence } from 'motion/react'`
- `import { GatewaySelectionAlgorithm }` from types
- `function Chevrons({ onUp, onDown })`
- `const chevronsDisabled = ...`
- `const [foldState, setFoldState] = useState<FoldState>(...)`
- `const expand = () => setFoldState(...)` / `collapse = ...`

These trigger `@typescript-eslint/no-unused-vars` and `TS6133`. The branch
ships with that lint/tscheck noise on purpose; **do not delete or rename
without checking the user first**. Their original purpose is documented in
[`NewBottomComponent.fold-logic.md`](./NewBottomComponent.fold-logic.md).

---

## Open follow-ups (per NYM-1385 + verbal feedback)

- **i18n**: the pill labels (`Auto`, `Fast`, `Mixnet`), the `Best server` /
  `Random` row labels, the `Loading...`-replacing Skeleton (no text needed),
  and `'Best server for my location'` / `'Random server'` placeholders are all
  hardcoded English. Add keys to `home.json` and use `useTranslation('home')`.
- **Library-driven reconnect**: NYM-1385 says mode changes should trigger an
  automatic reconnect from the library side, not from the app. We currently
  fire `invoke('set_vpn_mode', ...)` without manually toggling
  connect/disconnect — verify with the daemon team that this is sufficient,
  and adjust if not.
- **Accessibility (WCAG 2.1)**: pill buttons need ARIA roles
  (`role="radiogroup"` + `role="radio"`/`aria-checked`), and the Skeleton
  state should set `aria-busy`.

---

## Commit log on this branch (most recent first)

1. `a5572afa7` — show Loading… (Skeleton) in NodeRow while gateway list is fetching;
   includes the `wg` / `mxEntry` / `mxExit` deps fix in the `gateway` memo.
2. `fd66dd687` — "1click enhancements" (squash of the initial work):
   build the 3-state `ModeToggle`, rewire `NodeRow`/`NodeLocation`/`Node` to
   the new auto-mode semantics, add Random + Best server quick picks,
   highlight active rows, defer entry mode flip to selection, strip stale
   highlights via `useNodeListData`, etc.

See `git log fd66dd687~1..HEAD -- nym-vpn-app/src/screens/home nym-vpn-app/src/screens/node nym-vpn-app/src/hooks/useNodeListData.ts nym-vpn-app/src/assets/icons/gateway-mode nym-vpn-app/src/i18n/en/home.json`
for the file-level diff.
