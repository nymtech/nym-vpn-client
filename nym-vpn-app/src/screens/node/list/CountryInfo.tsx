import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { UiCountry } from '../../../contexts';
import { FlagIcon, countryCode } from '../../../ui';

type CountryInfoProps = {
  country: UiCountry;
  name: string;
  gwCount: number;
};

const CountryInfo = ({ country, name, gwCount }: CountryInfoProps) => {
  const { t } = useTranslation('glossary');

  return (
    <div
      className={clsx(
        'flex flex-row items-center ml-2 gap-3 overflow-hidden',
        'select-none cursor-default',
      )}
      data-testid={`country-info-${country.code}`}
    >
      <FlagIcon
        code={country.code.toLowerCase() as countryCode}
        alt={country.code}
        className="h-6"
        data-testid={`country-flag-${country.code}`}
      />
      <div className="flex flex-col justify-center overflow-hidden pr-4">
        <div
          className={clsx('text-baltic-sea dark:text-white text-base truncate')}
          data-testid={`country-name-${country.code}`}
        >
          {name}
        </div>
        <div
          className="text-iron dark:text-bombay text-sm"
          data-testid={`country-server-count-${country.code}`}
        >
          {`${gwCount} ${t('server', { count: gwCount })}`}
        </div>
      </div>
    </div>
  );
};

export default CountryInfo;
