import clsx from 'clsx';
import { motion } from 'motion/react';

type Props = {
  children: React.ReactNode;
  className?: string;
  slideOrigin?: 'left' | 'right';
  'data-testid'?: string;
};

function PageAnim({
  children,
  className,
  slideOrigin = 'left',
  ...rest
}: Props) {
  const testId = rest['data-testid'] || 'page-animation';

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
      data-testid={testId}
      data-slide-origin={slideOrigin}
    >
      {children}
    </motion.div>
  );
}

export default PageAnim;
