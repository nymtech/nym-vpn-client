import clsx from 'clsx';

export type PulseDotProps = {
  color: 'cornflower' | 'red' | 'yellow';
};

function PulseDot({ color = 'cornflower' }: PulseDotProps) {
  const dotColor = () => {
    switch (color) {
      case 'cornflower':
        return 'bg-cornflower';
      case 'red':
        return 'bg-rouge-ecarlate';
      case 'yellow':
        return 'bg-[#f59e0b] dark:bg-king-nacho';
    }
  };

  return (
    <div
      className={clsx([
        'relative flex justify-center items-center',
        // use static pixel sizes for animated elements to avoid glitches
        // with the different UI scaling factors
        'h-[10px] w-[10px]',
      ])}
    >
      <div
        className={clsx(
          'animate-ping absolute h-full w-full rounded-full opacity-75',
          dotColor(),
        )}
      />
      <div
        className={clsx('relative rounded-full', 'h-[6px] w-[6px]', dotColor())}
      />
    </div>
  );
}

export default PulseDot;
