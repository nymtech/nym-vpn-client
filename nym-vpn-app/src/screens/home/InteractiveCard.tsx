import clsx from 'clsx';
import { motion, useAnimationControls } from 'motion/react';
import { useEffect, useLayoutEffect } from 'react';
import { useCardAnimation } from '../../contexts/CardAnimationContext';

export function InteractiveCard({
  children,
  className = '',
}: {
  children: React.ReactNode;
  className?: string;
}) {
  const controls = useAnimationControls();
  const { registerExit } = useCardAnimation();

  useLayoutEffect(() => {
    controls.start({ y: 0 });
  }, [controls]);

  useEffect(() => {
    registerExit(() => controls.start({ y: '100%' }));
    return () => registerExit(null);
  }, [controls, registerExit]);

  return (
    <div className="flex h-full flex-col justify-end overflow-hidden">
      <motion.div
        animate={controls}
        initial={{ y: '100%' }}
        style={{ originY: 1 }}
        transition={{
          duration: 0.15,
          ease: 'easeOut',
        }}
        className={clsx(
          'z-20 flex flex-col overflow-hidden rounded-2xl bg-white p-5 dark:bg-[#1d1d1f]',
          className,
        )}
      >
        <motion.div
          layout="position"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.15 }}
          className="flex flex-1 flex-col"
        >
          {children}
        </motion.div>
      </motion.div>
    </div>
  );
}
