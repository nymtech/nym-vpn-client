import clsx from 'clsx';
import { motion } from 'motion/react';
import { NymSplash } from '../assets';

function IntroSplash({ theme }: { theme: 'light' | 'dark' }) {
  return (
    <div className={clsx([theme === 'dark' && 'dark'])}>
      <motion.div
        className={clsx([
          'absolute z-200 flex h-full w-full min-w-44 items-center justify-center',
          'bg-surface-bg scroll-none overflow-hidden',
        ])}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.2, ease: 'easeOut' }}
      >
        <NymSplash className="fill-text-primary w-36 dark:fill-white" />
      </motion.div>
    </div>
  );
}

export default IntroSplash;
