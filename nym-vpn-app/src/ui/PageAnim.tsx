import clsx from 'clsx';
import { motion } from 'motion/react';

type Props = {
  children: React.ReactNode;
  className?: string;
  slideOrigin?: 'left' | 'right';
};

function PageAnim({ children, className, slideOrigin = 'left' }: Props) {
  return (
    <motion.div
      initial={{
        opacity: 0,
        translateX: slideOrigin === 'left' ? -6 : 6,
      }}
      animate={{
        opacity: 1,
        translateX: 0,
        transition: { duration: 0.15, ease: 'easeOut' },
      }}
      className={clsx([className])}
    >
      {children}
    </motion.div>
  );
}

export default PageAnim;
