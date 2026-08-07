import { useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { AnimatePresence, motion } from 'motion/react';
import { useShallow } from 'zustand/react/shallow';
import { useAppStore } from '../../store';
import {
  useI18nAccountState,
  useI18nError,
  useI18nProgressMsg,
  useI18nTunnelError,
} from '../../hooks';
import { setToString } from '../../util';
import ConnectionTimer from './ConnectionTimer';

// ─── Geometry ────────────────────────────────────────────────────────────────
const SIZE = 180;
const CX = SIZE / 2;
const STROKE = 6;
const R_OUTER = SIZE * 0.42;
const R_MIDDLE = R_OUTER - 14;
const R_INNER = R_MIDDLE - 14;
const RADII = [R_OUTER, R_MIDDLE, R_INNER] as const;
const CIRCS = RADII.map((r) => 2 * Math.PI * r);

// Sphere sits inside the innermost ring with a small gap
const SPHERE_INSET = CX - (R_INNER - STROKE - 6);
const SPHERE_SIZE = SIZE - SPHERE_INSET * 2;
const GLOW_SPREAD = SPHERE_SIZE * 0.55;

// ─── Progress → ring/half mapping ────────────────────────────────────────────
const PROGRESS_STEPS = {
  'resolving-api-addresses': { ring: 0, half: true },
  'awaiting-account-readiness': { ring: 0, half: false },
  'awaiting-credentials-availability': { ring: 0, half: false },
  'refreshing-gateways': { ring: 1, half: true },
  'selecting-gateways': { ring: 1, half: false },
  'registering-with-gateways': { ring: 2, half: true },
  'connecting-tunnel': { ring: 2, half: false },
} as const;

// ─── Colors ───────────────────────────────────────────────────────────────────
const TRACK = 'var(--nv-connection-arc-track)';
const FILL = 'var(--nv-brand-primary)';
const ERROR_CLR = 'var(--nv-status-error)';

// Delay (ms) before surfacing the transient `needs-relaxed-independence-criteria`
// error, so a quick auto-relax + reconnect (e.g. when switching servers while
// connected) doesn't flash the error UI for an error that resolves on its own.
const RELAXED_INDEPENDENCE_ERROR_DELAY = 1000;

type Phase =
  'disconnected' | 'connecting' | 'connected' | 'canceling' | 'error';

// ─────────────────────────────────────────────────────────────────────────────

export function TunnelState() {
  const { state, connectingState, progressMessages } = useAppStore(
    useShallow((s) => ({
      state: s.state,
      connectingState: s.connectingState,
      progressMessages: s.progressMessages,
      vpnMode: s.vpnMode,
    })),
  );

  const tunnelError = useAppStore((s) => s.tunnelError);
  const accountState = useAppStore((s) => s.accountState);
  const accountError = useAppStore((s) => s.accountError);
  const error = useAppStore((s) => s.error);

  const { tE } = useI18nError();
  const { tTE } = useI18nTunnelError();
  const { t: tA } = useI18nAccountState();
  const { t: tP } = useI18nProgressMsg();
  const { t } = useTranslation('home');

  // ─── Derived booleans ─────────────────────────────────────────────────────
  const isAccountError =
    accountState === 'max-device-reached' ||
    accountState === 'no-subscription' ||
    accountState === 'bandwidth-exceeded' ||
    accountState === 'pending-subscription' ||
    accountState === 'status-not-active' ||
    accountState === 'error';

  // ─── Throttle the transient relaxed-independence error ────────────────────
  // While connected, switching servers can make the daemon briefly surface
  // `needs-relaxed-independence-criteria` before the watcher auto-relaxes and
  // reconnects. Delay surfacing it.
  const isRelaxedIndependenceError =
    state === 'error' && tunnelError === 'needs-relaxed-independence-criteria';

  const [showRelaxedIndependenceError, setShowRelaxedIndependenceError] =
    useState(false);
  useEffect(() => {
    if (!isRelaxedIndependenceError) {
      setShowRelaxedIndependenceError(false);
      return;
    }
    const id = setTimeout(
      () => setShowRelaxedIndependenceError(true),
      RELAXED_INDEPENDENCE_ERROR_DELAY,
    );
    return () => clearTimeout(id);
  }, [isRelaxedIndependenceError]);

  const suppressRelaxedIndependenceError =
    isRelaxedIndependenceError && !showRelaxedIndependenceError;

  const isError =
    !suppressRelaxedIndependenceError &&
    (state === 'error' ||
      state === 'unknown' ||
      state === 'offline' ||
      state === 'offline-auto-reconnect');
  const isConnected = state === 'connected';
  const isConnecting = state === 'connecting';
  const isCanceling = state === 'disconnecting';

  const computedPhase: Phase = isError
    ? 'error'
    : isCanceling
      ? 'canceling'
      : isConnected
        ? 'connected'
        : isConnecting
          ? 'connecting'
          : 'disconnected';

  // While suppressing the transient error, keep showing the previous phase
  // (typically `connecting` during a server switch) so nothing flashes.
  const lastPhaseRef = useRef<Phase>('disconnected');
  const phase: Phase = suppressRelaxedIndependenceError
    ? lastPhaseRef.current
    : computedPhase;
  useEffect(() => {
    if (!suppressRelaxedIndependenceError) {
      lastPhaseRef.current = computedPhase;
    }
  }, [suppressRelaxedIndependenceError, computedPhase]);

  // ─── Ring targets based on current progress ───────────────────────────────
  const progress = connectingState?.progress;

  const ringTargets = useMemo((): number[] => {
    if (phase === 'connected') return [0, 0, 0];
    if (phase === 'error') return [0, 0, 0];
    if (phase !== 'connecting' || !progress) return [...CIRCS];
    const step = PROGRESS_STEPS[progress];
    if (!step) return [...CIRCS];
    return CIRCS.map((C, i) => {
      if (i < step.ring) return 0;
      if (i === step.ring) return step.half ? C / 2 : 0;
      return C;
    });
  }, [phase, progress]);

  // ─── Freeze offsets when entering canceling ──────────────────────────────
  // Updated whenever we're NOT in canceling, so when that phase begins
  // the ref already holds the last "live" position.
  const frozenOffsets = useRef<number[]>([...CIRCS]);
  useEffect(() => {
    if (!isCanceling) {
      frozenOffsets.current = ringTargets;
    }
  }, [isCanceling, ringTargets]);

  const effectiveOffsets = isCanceling ? frozenOffsets.current : ringTargets;

  // ─── Per-ring stroke appearance ───────────────────────────────────────────
  const strokeColor = phase === 'error' ? ERROR_CLR : FILL;
  const strokeOpacity =
    phase === 'canceling' ? 0.15 : phase === 'error' ? 0.6 : 1;

  const ringTransition =
    phase === 'canceling'
      ? 'stroke-opacity 600ms ease-in, stroke-dashoffset 0ms'
      : 'stroke-dashoffset 800ms cubic-bezier(.4,0,.2,1), stroke 200ms ease, stroke-opacity 200ms ease';

  // ─── Label text ───────────────────────────────────────────────────────────
  const label = useMemo((): string | null => {
    if (phase === 'disconnected') return t('not-protected');

    if (phase === 'connecting') {
      const prog = progress ?? progressMessages[progressMessages.length - 1];
      return prog ? tP(prog) : null;
    }
    if (phase === 'canceling') return tP('canceling');
    return null;
  }, [phase, progress, progressMessages, tP, t]);

  // ─── Error content ────────────────────────────────────────────────────────
  const getError = () => {
    if (state === 'offline') return <p>{t('offline-message')}</p>;
    if (state === 'offline-auto-reconnect')
      return <p>{t('offline-reconnect-message')}</p>;
    if (tunnelError)
      return <p data-testid="tunnel-specific-error">{tTE(tunnelError)}</p>;
    if (isAccountError) {
      const msg = accountError ? tE(accountError.key) : tA(accountState);
      return <p data-testid="account-specific-error">{msg}</p>;
    }
    if (error) {
      return (
        <>
          <p data-testid="tunnel-error-key">
            {error.key ? tE(error.key) : error.message}
          </p>
          {error.data && (
            <p className="text-left" data-testid="tunnel-error-data">
              {setToString(error.data)}
            </p>
          )}
        </>
      );
    }
    return null;
  };

  return (
    <div className="inline-flex flex-col items-center justify-center gap-2 rounded-lg p-3.5">
      {/* Ring area */}
      <div style={{ position: 'relative', width: SIZE, height: SIZE }}>
        {/* SVG rings: rotated so arc starts at top (12 o'clock) */}
        <svg
          width={SIZE}
          height={SIZE}
          viewBox={`0 0 ${SIZE} ${SIZE}`}
          style={{ transform: 'rotate(-90deg)', display: 'block' }}
        >
          {/* Track circles (always-visible gray guides) */}
          {RADII.map((r, i) => (
            <circle
              key={`track-${i}`}
              cx={CX}
              cy={CX}
              r={r}
              fill="none"
              strokeWidth={STROKE}
              strokeLinecap="round"
              style={{ stroke: TRACK }}
            />
          ))}

          {/* Animated fill arcs */}
          {RADII.map((r, i) => (
            <circle
              key={`fill-${i}`}
              cx={CX}
              cy={CX}
              r={r}
              fill="none"
              strokeOpacity={strokeOpacity}
              strokeWidth={STROKE}
              strokeLinecap="round"
              strokeDasharray={CIRCS[i]}
              strokeDashoffset={effectiveOffsets[i]}
              style={{ stroke: strokeColor, transition: ringTransition }}
            />
          ))}
        </svg>

        {/* Connected glow halo (behind sphere, fades in) */}
        {isConnected && (
          <motion.div
            initial={{ opacity: 0, scale: 0.7 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.32, ease: 'easeOut' }}
            style={{
              position: 'absolute',
              top: SPHERE_INSET - GLOW_SPREAD / 2,
              left: SPHERE_INSET - GLOW_SPREAD / 2,
              width: SPHERE_SIZE + GLOW_SPREAD,
              height: SPHERE_SIZE + GLOW_SPREAD,
              borderRadius: '50%',
              pointerEvents: 'none',
            }}
          />
        )}
      </div>

      <div className="flex w-full flex-col items-center">
        <ConnectionTimer />
        <div className="min-h-8 w-full">
          <AnimatePresence mode="wait">
            {label !== null && (
              <motion.div
                key={label}
                data-testid="connection-status-text"
                initial={{ opacity: 0, x: -12 }}
                animate={{ opacity: 1, x: 0 }}
                exit={{ opacity: 0, x: 12 }}
                transition={{ duration: 0.2, ease: 'easeOut' }}
                className="text-text-secondary flex items-center justify-center text-lg"
              >
                {label}
              </motion.div>
            )}
          </AnimatePresence>
          {phase === 'error' && (
            <div className="text-status-error text-center text-lg">
              {getError()}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
