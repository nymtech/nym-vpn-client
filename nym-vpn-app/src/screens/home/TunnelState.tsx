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
import { ScrambleIn } from '../../ui/ScrambleIn';
import { setToString } from '../../util';

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
  'refreshing-gateways': { ring: 1, half: true },
  'selecting-gateways': { ring: 1, half: false },
  'registering-with-gateways': { ring: 2, half: true },
  'connecting-tunnel': { ring: 2, half: false },
} as const;

// ─── Colors ───────────────────────────────────────────────────────────────────
const TRACK = 'rgba(255,255,255,0.15)';
const FILL_FAST = 'var(--color-malachite-200)';
const FILL_ANON = 'rgba(139,139,144,0.60)';
const ERROR_CLR = 'var(--color-error)';

type Phase =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'canceling'
  | 'error';

// ─────────────────────────────────────────────────────────────────────────────

export function TunnelState() {
  const { state, connectingState, progressMessages, vpnMode } = useAppStore(
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

  const isError =
    state === 'error' ||
    state === 'unknown' ||
    state === 'offline' ||
    state === 'offline-auto-reconnect';
  const isConnected = state === 'connected';
  const isConnecting = state === 'connecting';
  const isCanceling = state === 'disconnecting';

  const phase: Phase = isError
    ? 'error'
    : isCanceling
      ? 'canceling'
      : isConnected
        ? 'connected'
        : isConnecting
          ? 'connecting'
          : 'disconnected';

  // ─── Mode → sweep duration and fill color ─────────────────────────────────
  const isMixnet = vpnMode === 'mixnet';
  const sweepDur = isMixnet ? 1200 : 800;
  const fillColor = isMixnet ? FILL_ANON : FILL_FAST;

  // ─── Ring targets based on current progress ───────────────────────────────
  const progress = connectingState?.progress;

  const ringTargets = useMemo((): number[] => {
    if (phase === 'connected') return [0, 0, 0];
    if (phase !== 'connecting' || !progress) return [...CIRCS];
    const step = PROGRESS_STEPS[progress];
    if (!step) return [...CIRCS];
    return CIRCS.map((C, i) => {
      if (i < step.ring) return 0;
      if (i === step.ring) return step.half ? C / 2 : 0;
      return C;
    });
  }, [phase, progress]);

  // ─── Freeze offsets when entering error/canceling ────────────────────────
  // Updated whenever we're NOT in error/canceling, so when those phases begin
  // the ref already holds the last "live" position.
  const frozenOffsets = useRef<number[]>([...CIRCS]);
  useEffect(() => {
    if (!isError && !isCanceling) {
      frozenOffsets.current = ringTargets;
    }
  }, [isError, isCanceling, ringTargets]);

  const effectiveOffsets =
    isError || isCanceling ? frozenOffsets.current : ringTargets;

  // ─── Per-ring stroke appearance ───────────────────────────────────────────
  const strokeColor = phase === 'error' ? ERROR_CLR : fillColor;
  const strokeOpacity =
    phase === 'canceling' ? 0.15 : phase === 'error' ? 0.6 : 1;

  const ringTransition =
    phase === 'error'
      ? 'stroke 200ms ease, stroke-opacity 200ms ease'
      : phase === 'canceling'
        ? 'stroke-opacity 600ms ease-in, stroke-dashoffset 0ms'
        : `stroke-dashoffset ${sweepDur}ms cubic-bezier(.4,0,.2,1), stroke 200ms ease, stroke-opacity 200ms ease`;

  // ─── Sphere opacity ───────────────────────────────────────────────────────
  const sphereOpacity =
    phase === 'connected'
      ? 0.65
      : phase === 'connecting'
        ? 0.52
        : phase === 'error'
          ? 0.55
          : 0.42;

  // Bump animation when transitioning to connected
  const [bumpKey, setBumpKey] = useState(0);
  const prevPhase = useRef<Phase>(phase);
  useEffect(() => {
    if (prevPhase.current !== 'connected' && phase === 'connected') {
      setBumpKey((k) => k + 1);
    }
    prevPhase.current = phase;
  }, [phase]);

  // ─── Glow color ───────────────────────────────────────────────────────────
  const glowColor = isMixnet
    ? 'rgba(180,182,186,0.55)'
    : 'rgba(91,240,160,0.70)';
  const glowColorTransparent = isMixnet
    ? 'rgba(180,182,186,0)'
    : 'rgba(91,240,160,0)';

  // ─── Label text ───────────────────────────────────────────────────────────
  const label = useMemo((): string | null => {
    if (phase === 'connecting') {
      const prog = progress ?? progressMessages[progressMessages.length - 1];
      return prog ? tP(prog) : null;
    }
    if (phase === 'canceling') return tP('canceling');
    return null;
  }, [phase, progress, progressMessages, tP]);

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
    <div className="inline-flex flex-col items-center gap-2 rounded-lg p-3.5 h-full justify-center">
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
              stroke={TRACK}
              strokeWidth={STROKE}
              strokeLinecap="round"
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
              stroke={strokeColor}
              strokeOpacity={strokeOpacity}
              strokeWidth={STROKE}
              strokeLinecap="round"
              strokeDasharray={CIRCS[i]}
              strokeDashoffset={effectiveOffsets[i]}
              style={{ transition: ringTransition }}
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
              background: `radial-gradient(circle, ${glowColor} 0%, ${glowColorTransparent} 60%)`,
              pointerEvents: 'none',
            }}
          />
        )}

        {/* Central sphere — remounted on bumpKey to replay seal animation */}
        <div
          key={`sphere-${bumpKey}`}
          style={{
            position: 'absolute',
            top: SPHERE_INSET,
            left: SPHERE_INSET,
            width: SPHERE_SIZE,
            height: SPHERE_SIZE,
            borderRadius: '50%',
            background: 'radial-gradient(circle at 30% 30%, #2A2A2E, #050505)',
            opacity: sphereOpacity,
            transition: 'opacity 300ms ease-out, box-shadow 320ms ease-out',
            boxShadow: isConnected
              ? isMixnet
                ? `0 0 ${SIZE * 0.1}px rgba(180,182,186,0.35)`
                : `0 0 ${SIZE * 0.12}px rgba(91,240,160,0.55)`
              : 'none',
            animation: bumpKey > 0 ? 'ts2-seal 200ms ease-out' : undefined,
          }}
        />

        {/* Error tint overlay on sphere */}
        {phase === 'error' && (
          <div
            style={{
              position: 'absolute',
              top: SPHERE_INSET,
              left: SPHERE_INSET,
              width: SPHERE_SIZE,
              height: SPHERE_SIZE,
              borderRadius: '50%',
              background: 'rgba(231,62,20,0.08)',
              pointerEvents: 'none',
              transition: 'opacity 200ms ease',
            }}
          />
        )}
      </div>

      {/* Label / error text */}
      <div className="flex h-4 items-center justify-center">
        <AnimatePresence mode="wait">
          {label !== null && (
            <motion.div
              key={label}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.15 }}
            >
              <ScrambleIn
                text={label}
                className="text-[9px] text-[#8b8b90]"
                scrambledClassName="text-[9px] text-[#8b8b90]/50"
                scrambleSpeed={20}
              />
            </motion.div>
          )}
        </AnimatePresence>
        {phase === 'error' && (
          <div className="text-[9px] text-error text-center">{getError()}</div>
        )}
      </div>
    </div>
  );
}
