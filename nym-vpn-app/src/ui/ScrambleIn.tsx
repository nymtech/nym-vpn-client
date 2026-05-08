import {
  forwardRef,
  useCallback,
  useEffect,
  useImperativeHandle,
  useState,
} from 'react';

export type ScrambleInProps = {
  text: string;
  scrambleSpeed?: number;
  characters?: string;
  className?: string;
  scrambledClassName?: string;
  autoStart?: boolean;
  onStart?: () => void;
  onComplete?: () => void;
};

export type ScrambleInHandle = {
  start: () => void;
  reset: () => void;
};

function randomChar(characters: string) {
  return characters[Math.floor(Math.random() * characters.length)];
}

function shuffle(arr: number[]): number[] {
  const out = [...arr];
  for (let i = out.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}

export const ScrambleIn = forwardRef<ScrambleInHandle, ScrambleInProps>(
  (
    {
      text,
      scrambleSpeed = 30,
      characters = 'abcdefghijklmnopqrstuvwxyz!@#$%^&*()_+',
      className = '',
      scrambledClassName = '',
      autoStart = true,
      onStart,
      onComplete,
    },
    ref,
  ) => {
    // shuffledIndices[0..resolvedCount) are the positions already settled
    const [shuffledIndices, setShuffledIndices] = useState<number[]>([]);
    const [resolvedCount, setResolvedCount] = useState(0);
    const [scrambledChars, setScrambledChars] = useState<string[]>([]);
    const [isAnimating, setIsAnimating] = useState(false);

    const startAnimation = useCallback(() => {
      const indices = shuffle(Array.from({ length: text.length }, (_, i) => i));
      const chars = Array.from({ length: text.length }, () =>
        randomChar(characters),
      );
      setShuffledIndices(indices);
      setScrambledChars(chars);
      setResolvedCount(0);
      setIsAnimating(true);
      onStart?.();
    }, [text, characters, onStart]);

    const reset = useCallback(() => {
      setIsAnimating(false);
      setResolvedCount(0);
      setShuffledIndices([]);
      setScrambledChars([]);
    }, []);

    useImperativeHandle(ref, () => ({ start: startAnimation, reset }));

    useEffect(() => {
      if (autoStart) startAnimation();
    }, [autoStart, startAnimation]);

    useEffect(() => {
      if (!isAnimating) return;
      if (resolvedCount >= text.length) {
        setIsAnimating(false);
        onComplete?.();
        return;
      }

      const timeout = setTimeout(() => {
        setResolvedCount((prev) => prev + 1);
        setScrambledChars(
          Array.from({ length: text.length }, () => randomChar(characters)),
        );
      }, scrambleSpeed);

      return () => clearTimeout(timeout);
    }, [
      isAnimating,
      resolvedCount,
      text,
      characters,
      scrambleSpeed,
      onComplete,
    ]);

    const resolvedSet = new Set(shuffledIndices.slice(0, resolvedCount));

    return (
      <>
        <span className="sr-only">{text}</span>
        <span className="inline-block whitespace-pre-wrap" aria-hidden="true">
          {Array.from(text).map((char, i) => {
            const settled = resolvedSet.has(i);
            return (
              <span
                key={i}
                className={settled ? className : scrambledClassName}
              >
                {settled ? char : (scrambledChars[i] ?? char)}
              </span>
            );
          })}
        </span>
      </>
    );
  },
);
ScrambleIn.displayName = 'ScrambleIn';
