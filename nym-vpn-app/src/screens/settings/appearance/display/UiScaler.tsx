import { ChangeEvent, useEffect, useState } from 'react';
import clsx from 'clsx';
import { DefaultRootFontSize } from '../../../../constants';
import { useMainDispatch, useMainState } from '../../../../contexts';
import { kvSet } from '../../../../kvStore';
import { StateDispatch } from '../../../../types';

function UiScaler() {
  const [slideValue, setSlideValue] = useState(DefaultRootFontSize);
  const dispatch = useMainDispatch() as StateDispatch;
  const { rootFontSize } = useMainState();

  useEffect(() => {
    setSlideValue(rootFontSize);
  }, [rootFontSize]);

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    setSlideValue(parseInt(e.target.value));
    dispatch({ type: 'set-root-font-size', size: slideValue });
  };

  const setNewFontSize = () => {
    document.documentElement.style.fontSize = `${slideValue}px`;
    dispatch({ type: 'set-root-font-size', size: slideValue });
    kvSet('UiRootFontSize', slideValue);
  };

  return (
    <div
      className={clsx([
        'flex flex-row justify-between items-center gap-10',
        'bg-white dark:bg-octave',
        'px-6 py-4 rounded-lg',
      ])}
    >
      <p className="text-base text-baltic-sea dark:text-mercury-pinkish flex-nowrap select-none">
        {slideValue}
      </p>
      <input
        type="range"
        min="8"
        max="20"
        value={slideValue}
        onChange={handleChange}
        onMouseUp={setNewFontSize}
        onKeyUp={setNewFontSize}
        className="range flex flex-1 accent-malachite"
      />
    </div>
  );
}

export default UiScaler;
