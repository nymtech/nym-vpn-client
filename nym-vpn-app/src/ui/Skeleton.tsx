import clsx from 'clsx';

export type SkeletonProps = {
  className?: string;
  rounded?: boolean | 'full';
};

function Skeleton({ className, rounded = true }: SkeletonProps) {
  const getRoundedClass = () => {
    if (rounded === false) return '';
    if (rounded === 'full') return 'rounded-full';
    return 'rounded';
  };

  return (
    <div
      className={clsx([
        'animate-pulse',
        'bg-iron dark:bg-bombay',
        getRoundedClass(),
        className,
      ])}
    />
  );
}

export default Skeleton;
