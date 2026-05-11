import { motion } from 'motion/react';
import { ReactNode } from 'react';

export const PanelContent = ({
  children,
  animate = false,
}: {
  children: ReactNode;
  animate?: boolean;
}) => {
  return (
    <motion.div
      initial={animate && { opacity: 0, translateY: -4 }}
      animate={animate && { opacity: 1, translateY: 0 }}
      transition={animate ? { duration: 0.1, ease: 'easeIn' } : undefined}
      className="group flex flex-col"
    >
      {children}
    </motion.div>
  );
};
