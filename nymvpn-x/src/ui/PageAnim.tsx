import clsx from 'clsx';
import AnimateIn from './AnimateIn';

type Props = {
  children: React.ReactNode;
  className?: string;
};

function PageAnim({ children, className }: Props) {
  return (
    <AnimateIn
      from="opacity-0 -translate-x-4"
      to="opacity-100 translate-x-0"
      duration={150}
      className={clsx([className])}
    >
      {children}
    </AnimateIn>
  );
}

export default PageAnim;
