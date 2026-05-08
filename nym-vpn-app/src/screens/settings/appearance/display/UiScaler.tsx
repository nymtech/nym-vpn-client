import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { DefaultRootFontSize } from '../../../../constants';
import { dispatch, useMainState } from '../../../../store';
import { kvSet } from '../../../../kvStore';
import { Slider } from '../../../../ui';

function UiScaler() {
  const { t } = useTranslation('display');
  const [slideValue, setSlideValue] = useState(DefaultRootFontSize);
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
    kvSet('ui-root-font-size', size);
  };

  return (
    <div
      className={clsx([
        'flex flex-row items-center justify-between gap-10',
        'dark:bg-charcoal bg-white',
        'rounded-lg px-6 py-5',
      ])}
      data-testid="ui-scaler-container"
    >
      <p
        className="text-text-primary flex-nowrap text-base select-none"
        data-testid="ui-scaler-value"
      >
        {slideValue}
      </p>
      <Slider
        value={slideValue}
        step={1}
        min={8}
        max={20}
        onChange={handleChange}
        onValueCommitted={handleFinalChange}
        ariaLabel={t('zoom-section-title')}
      />
    </div>
  );
}

export default UiScaler;
