import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ButtonNew, TextInput } from '../../../../ui';
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
      <div className="h-full flex-1">
        <TextInput
          color="gray"
          value={value}
          placeholder={defaultValue}
          disabled={disabled}
          onChange={handleChange}
        />
        {error && <p className="text-aphrodisiac mt-2 text-xs">{error}</p>}
      </div>
      <div className="h-full">
        <ButtonNew variant="outlined" disabled={disabled} onClick={handleReset}>
          {t('app-proxy.reset-to-default')}
        </ButtonNew>
      </div>
    </div>
  );
}

export default ProxyPortInput;
