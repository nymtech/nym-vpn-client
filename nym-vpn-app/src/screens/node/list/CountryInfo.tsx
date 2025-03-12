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
    >
      <FlagIcon
        code={country.code.toLowerCase() as countryCode}
        alt={country.code}
        className="h-6"
      />
      <div className="flex flex-col justify-center overflow-hidden pr-4">
        <div
          className={clsx('text-baltic-sea dark:text-white text-base truncate')}
        >
          {name}
        </div>
        <div className="text-dim-gray dark:text-bombay text-sm">{`${gwCount} ${t('server', { count: gwCount })}`}</div>
      </div>
    </div>
  );
};

export default CountryInfo;
