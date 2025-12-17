import clsx from 'clsx';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button, TextInput } from '../../../../ui';
import { portRegex } from '../utils';

type MyInputProps = {
  value: string;
  defaultValue: string;
  disabled: boolean;
  onChange: (value: string, valid: boolean) => void;
};

function ProxyPortInput({
  value,
  defaultValue,
  disabled,
  onChange,
}: MyInputProps) {
  const [error, setError] = useState<string | null>(null);
  const { t } = useTranslation('settings');

  const handleChange = (value: string) => {
    const valid = portRegex.test(value);
    onChange(value, valid);
    setError(valid ? null : t('app-proxy.invalid-port'));
  };

  const handleReset = () => {
    onChange(defaultValue, true);
    setError(null);
  };

  return (
    <div className="flex flex-row gap-2">
      <div className="flex-1 h-full">
        <TextInput
          color="gray"
          value={value}
          placeholder={defaultValue}
          disabled={disabled}
          label={t('app-proxy.listen-port')}
          onChange={handleChange}
        />
        {error && <p className="mt-2 text-xs text-aphrodisiac">{error}</p>}
      </div>
      <div className="h-full">
        <Button
          outline
          color="gray"
          disabled={disabled}
          onClick={handleReset}
          className="group border border-iron dark:border-bombay hover:ring-0! dark:hover:ring-0! py-2.5!"
        >
          <span
            className={clsx(
              'text-lg text-black dark:text-white',
              !disabled &&
                'group-hover:text-black/50 dark:group-hover:text-white/80',
            )}
          >
            {t('app-proxy.reset-to-default')}
          </span>
        </Button>
      </div>
    </div>
  );
}

export default ProxyPortInput;
