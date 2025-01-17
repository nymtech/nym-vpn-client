import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { DefaultRootFontSize } from '../../../../constants';
import { useMainDispatch, useMainState } from '../../../../contexts';
import { kvSet } from '../../../../kvStore';
import { StateDispatch } from '../../../../types';
import { Slider } from '../../../../ui';

function UiScaler() {
  const [slideValue, setSlideValue] = useState(DefaultRootFontSize);
  const dispatch = useMainDispatch() as StateDispatch;
  const { rootFontSize } = useMainState();

  useEffect(() => {
    setSlideValue(rootFontSize);
  }, [rootFontSize]);

  const handleChange = (size: number) => {
    setSlideValue(size);
    dispatch({ type: 'set-root-font-size', size });
  };

  const handleFinalChange = (size: number) => {
    document.documentElement.style.fontSize = `${size}px`;
    dispatch({ type: 'set-root-font-size', size });
    kvSet('UiRootFontSize', size);
  };

  return (
    <div
      className={clsx([
        'flex flex-row justify-between items-center gap-10',
        'bg-white dark:bg-octave',
        'px-6 py-5 rounded-lg',
      ])}
    >
      <p className="text-base text-baltic-sea dark:text-mercury-pinkish flex-nowrap select-none">
        {slideValue}
      </p>
      <Slider
        value={slideValue}
        step={1}
        min={8}
        max={20}
        onChange={handleChange}
        onCommit={handleFinalChange}
      />
    </div>
  );
}

export default UiScaler;
