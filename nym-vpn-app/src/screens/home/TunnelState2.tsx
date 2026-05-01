import {
  forwardRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useMemo,
  useState,
} from 'react';
import { useTranslation } from 'react-i18next';
import { AnimatePresence, motion } from 'motion/react';
import { useShallow } from 'zustand/react/shallow';
import { useAppStore } from '../../store';
import { useI18nProgressMsg } from '../../hooks';

// Inlined from fancycomponents.dev/docs/components/text/scramble-in

type ScrambleInProps = {
  text: string;
  scrambleSpeed?: number;
  scrambledLetterCount?: number;
  characters?: string;
  className?: string;
  scrambledClassName?: string;
  autoStart?: boolean;
  onStart?: () => void;
  onComplete?: () => void;
};

type ScrambleInHandle = {
  start: () => void;
  reset: () => void;
};

const ScrambleIn = forwardRef<ScrambleInHandle, ScrambleInProps>(
  (
    {
      text,
      scrambleSpeed = 50,
      scrambledLetterCount = 2,
      characters = 'abcdefghijklmnopqrstuvwxyz!@#$%^&*()_+',
      className = '',
      scrambledClassName = '',
      autoStart = true,
      onStart,
      onComplete,
    },
    ref,
  ) => {
    const [displayText, setDisplayText] = useState('');
    const [isAnimating, setIsAnimating] = useState(false);
    const [visibleLetterCount, setVisibleLetterCount] = useState(0);
    const [scrambleOffset, setScrambleOffset] = useState(0);

    const startAnimation = useCallback(() => {
      setIsAnimating(true);
      setVisibleLetterCount(0);
      setScrambleOffset(0);
      onStart?.();
    }, [onStart]);

    const reset = useCallback(() => {
      setIsAnimating(false);
      setVisibleLetterCount(0);
      setScrambleOffset(0);
      setDisplayText('');
    }, []);

    useImperativeHandle(ref, () => ({ start: startAnimation, reset }));

    useEffect(() => {
      if (autoStart) startAnimation();
    }, [autoStart, startAnimation]);

    useEffect(() => {
      let interval: ReturnType<typeof setInterval>;
      if (isAnimating) {
        interval = setInterval(() => {
          if (visibleLetterCount < text.length) {
            setVisibleLetterCount((prev) => prev + 1);
          } else if (scrambleOffset < scrambledLetterCount) {
            setScrambleOffset((prev) => prev + 1);
          } else {
            clearInterval(interval);
            setIsAnimating(false);
            onComplete?.();
          }

          const remaining = Math.max(0, text.length - visibleLetterCount);
          const scrambleCount = Math.min(remaining, scrambledLetterCount);
          const scrambled = Array(scrambleCount)
            .fill(0)
            .map(
              () => characters[Math.floor(Math.random() * characters.length)],
            )
            .join('');
          setDisplayText(text.slice(0, visibleLetterCount) + scrambled);
        }, scrambleSpeed);
      }
      return () => {
        if (interval) clearInterval(interval);
      };
    }, [
      isAnimating,
      text,
      visibleLetterCount,
      scrambleOffset,
      scrambledLetterCount,
      characters,
      scrambleSpeed,
      onComplete,
    ]);

    return (
      <>
        <span className="sr-only">{text}</span>
        <span className="inline-block whitespace-pre-wrap" aria-hidden="true">
          <span className={className}>
            {displayText.slice(0, visibleLetterCount)}
          </span>
          <span className={scrambledClassName}>
            {displayText.slice(visibleLetterCount)}
          </span>
        </span>
      </>
    );
  },
);
ScrambleIn.displayName = 'ScrambleIn';

// ---

// inset (%) — outermost (0) to innermost (2)
const RINGS = [{ inset: 12.5 }, { inset: 21.25 }, { inset: 30 }] as const;

const RING_GRAY = 'rgba(255,255,255,0.15)';
const RING_GREEN = 'var(--color-malachite-200)';
const RING_RED = 'var(--color-error)';
// How long each ring stays lit during the chase animation
const RING_PHASE_MS = 600;

export function TunnelState2() {
  const { state, connectingState, progressMessages } = useAppStore(
    useShallow((s) => ({
      state: s.state,
      connectingState: s.connectingState,
      progressMessages: s.progressMessages,
    })),
  );

  const { t: tP } = useI18nProgressMsg();
  const { t } = useTranslation('home');

  const [animPhase, setAnimPhase] = useState(0);

  // Reset the chase phase whenever the connection state changes
  useEffect(() => {
    setAnimPhase(0);
  }, [state]);

  // Advance the active ring index while connecting or disconnecting
  useEffect(() => {
    if (state !== 'connecting' && state !== 'disconnecting') return;
    const id = setInterval(
      () => setAnimPhase((p) => (p + 1) % 3),
      RING_PHASE_MS,
    );
    return () => clearInterval(id);
  }, [state]);

  const isError =
    state === 'error' ||
    state === 'unknown' ||
    state === 'offline' ||
    state === 'offline-auto-reconnect';

  function getRingColor(i: number): string {
    if (isError) return RING_RED;
    if (state === 'connected') return RING_GREEN;
    // connecting: outer(0) → middle(1) → inner(2)
    if (state === 'connecting') return i === animPhase ? RING_GREEN : RING_GRAY;
    // disconnecting: inner(2) → middle(1) → outer(0)
    if (state === 'disconnecting')
      return 2 - i === animPhase ? RING_GREEN : RING_GRAY;
    return RING_GRAY;
  }

  const label = useMemo((): string | null => {
    switch (state) {
      case 'connecting': {
        const progress =
          connectingState?.progress ??
          progressMessages[progressMessages.length - 1];
        return progress ? tP(progress) : null;
      }
      case 'disconnecting':
        return tP('canceling');
      case 'offline':
        return t('offline-message');
      case 'offline-auto-reconnect':
        return t('offline-reconnect-message');
      default:
        return null;
    }
  }, [state, connectingState, progressMessages, tP, t]);

  return (
    <div className="inline-flex flex-col items-center gap-2 rounded-lg bg- p-3.5 h-full justify-center">
      {/* Ring visualization */}
      <div className="relative max-h-[180px] max-w-[180px] h-full w-full">
        {RINGS.map((ring, i) => (
          <div
            key={i}
            className="absolute rounded-full border-[6px] transition-colors duration-250"
            style={{ inset: `${ring.inset}%`, borderColor: getRingColor(i) }}
          />
        ))}
      </div>

      {/* Scramble-in text label */}
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
                className={`text-[9px] ${isError ? 'text-error' : 'text-[#8b8b90]'}`}
                scrambledClassName={`text-[9px] ${isError ? 'text-error/50' : 'text-[#8b8b90]/50'}`}
                scrambleSpeed={35}
                scrambledLetterCount={3}
              />
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}
