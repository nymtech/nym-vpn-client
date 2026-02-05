import clsx from 'clsx';
import { motion } from 'motion/react';
import { useTranslation } from 'react-i18next';

type Props = {
  children: React.ReactNode;
  className?: string;
  slideOrigin?: 'left' | 'right';
  'data-testid'?: string;
};

function PageAnim({ children, className, slideOrigin, ...rest }: Props) {
  const { i18n } = useTranslation();
  const testId = rest['data-testid'] || 'page-animation';

  const origin = slideOrigin ?? (i18n.dir() === 'rtl' ? 'right' : 'left');

  return (
    <motion.div
      initial={{
        opacity: 0,
        translateX: origin === 'left' ? -6 : 6,
      }}
      animate={{
        opacity: 1,
        translateX: 0,
        transition: { duration: 0.15, ease: 'easeOut' },
      }}
      className={clsx([className])}
      data-testid={testId}
      data-test-slide-origin={slideOrigin}
    >
      {children}
    </motion.div>
  );
}

export default PageAnim;
