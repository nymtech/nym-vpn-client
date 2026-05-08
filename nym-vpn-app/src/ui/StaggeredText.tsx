import clsx from 'clsx';
import { motion, useInView } from 'framer-motion';
import { FC, useRef } from 'react';

type TextStaggeredFadeProps = {
  text: string;
  className?: string;
};

export const StaggeredText: FC<TextStaggeredFadeProps> = ({
  text,
  className = '',
}) => {
  const variants = {
    hidden: { opacity: 0 },
    show: (i: number) => ({
      y: 0,
      opacity: 1,
      transition: { delay: i * 0.005 },
    }),
  };

  const letters = text.split('');
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true });

  return (
    <motion.p
      ref={ref}
      initial="hidden"
      animate={isInView ? 'show' : ''}
      variants={variants}
      viewport={{ once: true }}
      className={clsx(className)}
    >
      {letters.map((word, i) => (
        <motion.span key={`${word}-${i}`} variants={variants} custom={i}>
          {word}
        </motion.span>
      ))}
    </motion.p>
  );
};
