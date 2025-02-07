import clsx from 'clsx';

export type PulseDotProps = {
  color: 'cornflower' | 'red';
};

function PulseDot({ color }: PulseDotProps) {
  const dotColor =
    color === 'cornflower' ? 'bg-cornflower' : 'bg-rouge-ecarlate';
  return (
    <div
      className={clsx([
        'relative flex justify-center items-center',
        // use static pixel sizes for animated elements to avoid glitches
        // with the different UI scaling factors
        'h-[12px] w-[12px]',
      ])}
    >
      <div
        className={clsx(
          'animate-ping absolute h-full w-full rounded-full opacity-75',
          dotColor,
        )}
      />
      <div
        className={clsx('relative rounded-full', 'h-[8px] w-[8px]', dotColor)}
      />
    </div>
  );
}

export default PulseDot;
