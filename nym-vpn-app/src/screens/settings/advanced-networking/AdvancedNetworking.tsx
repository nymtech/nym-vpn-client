import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { type } from '@tauri-apps/plugin-os';
import { CardSwitch, PageAnim, SettingsMenuCardBig, Slider } from '../../../ui';
import { dispatch, useMainState } from '../../../store';
import { useToast } from '../../../hooks';

const MIN_IPV6_MTU = 1280;
const ETHERNET_V2_MTU = 1500;
const WG_TUNNEL_OVERHEAD = 80;

// How long the user must stop interacting before we send the change to the
// daemon. Each reconnect tears down WireGuard sockets, so we coalesce rapid
// slider/toggle changes into a single apply.
const APPLY_DEBOUNCE_MS = 1000;

/**
 * Compute the maximum user-selectable exit MTU for the current OS + VPN mode.
 *
 * Mirrors the constants in `nym-vpn-lib/.../two_hop_config.rs` and
 * `tunnel_monitor.rs::DEFAULT_TUN_MTU`.
 */
function computeMaxMtu(
  os: ReturnType<typeof type>,
  vpnMode: 'wg' | 'mixnet',
): number {
  const isMobileLike = os === 'ios' || os === 'android';

  if (vpnMode === 'wg') {
    // WireGuard two-hop: exit ceiling = ethernet - 2x overhead, mobile = min v6
    return isMobileLike
      ? MIN_IPV6_MTU
      : ETHERNET_V2_MTU - WG_TUNNEL_OVERHEAD * 2;
  }
  // Mixnet: single TUN interface, ceiling = ethernet, mobile = min v6
  return isMobileLike ? MIN_IPV6_MTU : ETHERNET_V2_MTU;
}

function AdvancedNetworking() {
  const { mtu, vpnMode, state } = useMainState();
  const { add } = useToast();
  const { t } = useTranslation('settings');

  const os = type();
  const maxMtu = useMemo(() => computeMaxMtu(os, vpnMode), [os, vpnMode]);
  const minMtu = MIN_IPV6_MTU;

  const enabled = mtu !== null;

  // Locally remember the last value so toggling off doesn't lose the user's
  // chosen number — when the toggle flips back on we reuse it.
  const [lastValue, setLastValue] = useState<number>(mtu ?? maxMtu);
  useEffect(() => {
    if (mtu !== null) setLastValue(mtu);
  }, [mtu]);

  // Clamp the displayed value when the cap drops (e.g. user switched mode).
  const clampedValue = Math.min(Math.max(lastValue, minMtu), maxMtu);

  // Debounce + no-op detection. `pendingRef` is the value the user wants the
  // daemon to be at; `lastAppliedRef` is what the daemon last confirmed.
  // The timer fires APPLY_DEBOUNCE_MS after the last interaction.
  const pendingRef = useRef<number | null>(mtu);
  const lastAppliedRef = useRef<number | null>(mtu);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Resync refs when `mtu` changes from outside our optimistic flow — for
  // example when the daemon pushes a config-changed event because another
  // client (CLI, second app instance) edited the config. We can identify
  // "external" updates because they don't match what we just scheduled. If
  // we have a pending apply timer, cancel it: the daemon's current state
  // is now authoritative.
  useEffect(() => {
    if (mtu === pendingRef.current) return;
    pendingRef.current = mtu;
    lastAppliedRef.current = mtu;
    if (timerRef.current !== null) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, [mtu]);

  // On unmount with a pending change, fire-and-forget the RPC so the daemon
  // catches up with the optimistic UI we already committed to global state.
  // If the call fails after we've gone, roll back the global state — the
  // dispatch reference is stable, so this still works.
  useEffect(
    () => () => {
      if (timerRef.current === null) return;
      clearTimeout(timerRef.current);
      timerRef.current = null;
      const target = pendingRef.current;
      const previous = lastAppliedRef.current;
      if (target === previous) return;
      invoke('set_mtu', { mtu: target })
        .then(() => {
          lastAppliedRef.current = target;
        })
        .catch((err: unknown) => {
          console.error(
            '[advanced-networking] flush-on-unmount failed; rolling back',
            err,
          );
          dispatch({ type: 'set-mtu', mtu: previous });
        });
    },
    [],
  );

  const notifyApplied = useCallback(
    (isEnabled: boolean) => {
      if (state == 'connected' || state == 'connecting') {
        add({
          id: `mtu-switch-${isEnabled}`,
          title: t(
            isEnabled
              ? 'advanced-networking.mtu.snackbar-switch-on'
              : 'advanced-networking.mtu.snackbar-switch-off',
          ),
          type: 'info',
        });
      }
    },
    [state, t, add],
  );

  const scheduleApply = useCallback(
    (next: number | null) => {
      pendingRef.current = next;
      if (timerRef.current !== null) clearTimeout(timerRef.current);
      timerRef.current = setTimeout(async () => {
        timerRef.current = null;
        const target = pendingRef.current;
        const previous = lastAppliedRef.current;
        // No-op if the user ended up where they started — covers
        // toggle-off-then-on within 1s and slider-bounce-back.
        if (target === previous) return;
        try {
          await invoke('set_mtu', { mtu: target });
          lastAppliedRef.current = target;
          notifyApplied(target !== null);
        } catch (error) {
          console.error('[advanced-networking] set_mtu error', error);
          // Roll back the optimistic dispatch so the UI doesn't lie about
          // a value the daemon never accepted.
          dispatch({ type: 'set-mtu', mtu: previous });
          add({
            id: 'mtu-error',
            title: t('advanced-networking.mtu.errors.failed'),
            type: 'error',
          });
        }
      }, APPLY_DEBOUNCE_MS);
    },
    [notifyApplied, add, t],
  );

  const onToggle = () => {
    const next = enabled ? null : clampedValue;
    // Optimistic UI update so toggle/slider react instantly; the actual
    // RPC fires after debounce.
    dispatch({ type: 'set-mtu', mtu: next });
    scheduleApply(next);
  };

  // Live drag updates only the local "last value" for snappy UI feedback —
  // no debounce reset, no RPC.
  const onSliderChange = (value: number) => {
    setLastValue(value);
  };

  const onSliderCommitted = (value: number) => {
    if (!enabled) return;
    setLastValue(value);
    dispatch({ type: 'set-mtu', mtu: value });
    scheduleApply(value);
  };

  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-6 select-none">
      <div className="text-iron dark:text-bombay">
        {t('advanced-networking.intro')}
      </div>
      <SettingsMenuCardBig
        header={
          <CardSwitch
            header={t('advanced-networking.mtu.label')}
            subheader={t('advanced-networking.mtu.warning')}
            subheaderColor="king-nacho"
            checked={enabled}
            onClick={onToggle}
          />
        }
      >
        <div className="flex flex-col gap-4">
          <p className="text-iron dark:text-bombay text-sm whitespace-pre-line">
            {t('advanced-networking.mtu.content')}
          </p>
          {enabled ? (
            <div>
              <div className="text-iron dark:text-bombay mb-2 flex justify-between text-sm">
                <span>{minMtu}</span>
                <span className="text-cornflower">
                  {`${t('advanced-networking.mtu.current')}: ${clampedValue}`}
                </span>
                <span>{maxMtu}</span>
              </div>
              <Slider
                value={clampedValue}
                onChange={onSliderChange}
                onValueCommitted={onSliderCommitted}
                min={minMtu}
                max={maxMtu}
                step={1}
                ariaLabel={t('advanced-networking.mtu.label')}
              />
            </div>
          ) : (
            <p className="text-iron dark:text-bombay text-sm">
              {t('advanced-networking.mtu.platform-default')}
            </p>
          )}
        </div>
      </SettingsMenuCardBig>
    </PageAnim>
  );
}

export default AdvancedNetworking;
