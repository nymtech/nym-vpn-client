import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { Button, TextInput } from '../../../../ui';
import { portRegex } from '../utils';

type MyInputProps = {
  value: string;
  defaultValue: string;
  disabled: boolean;
  onChange: (value: string) => void;
};

function ProxyPortInput({
  value,
  defaultValue,
  disabled,
  onChange,
}: MyInputProps) {
  const { t } = useTranslation('settings');

  const handleChange = (value: string) => {
    if (portRegex.test(value) || value === '') {
      onChange(value);
    }
  };

  const handleReset = () => {
    onChange(defaultValue);
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
