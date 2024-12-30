import React, { useEffect, useState } from 'react';
import clsx from 'clsx';

// Utility Component for Animations with TailwindCSS
// taken from https://gist.github.com/johnpolacek/c7ddd607a4d5dbf43f38ae7266f6de18
// https://animate-in.vercel.app/
const AnimateIn = ({
  children,
  delay = 0,
  duration = 150,
  className = '',
  from,
  to,
  style,
  as = 'div',
}: {
  from: string;
  to: string;
  children?: React.ReactNode;
  delay?: number;
  duration?: number;
  className?: string;
  style?: React.CSSProperties;
  as?: 'div' | 'nav';
}) => {
  const [animate, setAnimate] = useState(from);
  const [prefersReducedMotion, setPrefersReducedMotion] = useState(false);

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');

    const mediaQueryChangeHandler = (e: MediaQueryListEvent) => {
      setPrefersReducedMotion(e.matches);
    };

    setPrefersReducedMotion(mediaQuery.matches);
    mediaQuery.addEventListener('change', mediaQueryChangeHandler);

    return () => {
      mediaQuery.removeEventListener('change', mediaQueryChangeHandler);
    };
  }, []);

  useEffect(() => {
    if (prefersReducedMotion) {
      // If the user prefers reduced motion, skip the animation
      setAnimate(to);
      return;
    }

    const timer = setTimeout(() => {
      setAnimate(to);
    }, delay);

    return () => clearTimeout(timer);
  }, [delay, to, prefersReducedMotion]);

  return React.createElement(
    as,
    {
      className: clsx('transition-all ease-in-out', className, animate),
      style: {
        transitionDuration: prefersReducedMotion ? '0ms' : `${duration}ms`,
        ...style,
      },
    },
    children,
  );
};

export default AnimateIn;
