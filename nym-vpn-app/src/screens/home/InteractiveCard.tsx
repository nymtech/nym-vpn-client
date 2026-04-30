import clsx from 'clsx';
import { motion } from 'motion/react';

export function InteractiveCard({
  children,
  className = '',
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div className="h-full flex justify-end flex-col overflow-hidden">
      <motion.div
        initial={{ y: '100%' }}
        animate={{ y: 0 }}
        exit={{ y: '100%' }}
        transition={{
          type: 'spring',
          stiffness: 280,
          damping: 28,
        }}
        className={clsx(
          'z-20 bg-white dark:bg-[#1d1d1f] rounded-2xl flex flex-col p-5',
          className,
        )}
      >
        {children}
      </motion.div>
    </div>
  );
}
