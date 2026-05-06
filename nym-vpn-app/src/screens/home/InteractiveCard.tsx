import clsx from 'clsx';
import { motion, useAnimationControls } from 'motion/react';
import { useEffect } from 'react';
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

  useEffect(() => {
    controls.start({ y: 0 });
  }, [controls]);

  useEffect(() => {
    registerExit(() => controls.start({ y: '100%' }));
    return () => registerExit(null);
  }, [controls, registerExit]);

  return (
    <div className="h-full flex justify-end flex-col overflow-hidden">
      <motion.div
        layout
        animate={controls}
        initial={{ y: '100%' }}
        style={{ originY: 1 }}
        transition={{
          duration: 0.15,
          ease: 'easeOut',
        }}
        className={clsx(
          'z-20 bg-white dark:bg-[#1d1d1f] rounded-2xl flex flex-col p-5 overflow-hidden',
          className,
        )}
      >
        <motion.div
          layout="position"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.15 }}
          className="flex flex-col flex-1"
        >
          {children}
        </motion.div>
      </motion.div>
    </div>
  );
}
