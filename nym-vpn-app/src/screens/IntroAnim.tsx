import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { AnimatePresence, motion } from 'motion/react';
import { DotLottieReact } from '@lottiefiles/dotlottie-react';

let initialized = false;
const splashDuration = 1000; // 1s, duration of the animation

function IntroAnim({ theme }: { theme: 'light' | 'dark' }) {
  const [completed, setCompleted] = useState(false);

  useEffect(() => {
    if (initialized) {
      return;
    }
    initialized = true;

    setTimeout(() => {
      setCompleted(true);
    }, splashDuration);
  }, []);

  return (
    <div className={clsx([theme === 'dark' && 'dark'])}>
      <AnimatePresence>
        {!completed && (
          <motion.div
            className={clsx([
              'absolute z-200 flex h-full w-full min-w-44 items-center justify-center',
              'bg-faded-lavender dark:bg-ash scroll-none overflow-hidden',
            ])}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 5 }}
            transition={{ duration: 0.2, ease: 'easeOut' }}
          >
            <DotLottieReact
              src="/animations/splash.json"
              autoplay
              loop={false}
            />
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default IntroAnim;
