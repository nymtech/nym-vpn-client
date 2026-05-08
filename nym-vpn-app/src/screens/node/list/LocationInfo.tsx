import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { UiCountry, UiRegion } from '../../../types/node';
import { FlagIcon, countryCode } from '../../../ui';

type LocationInfoProps = {
  node: UiCountry | UiRegion;
  name: string;
  gwCount: number;
};

const LocationInfo = ({ node, name, gwCount }: LocationInfoProps) => {
  const { t } = useTranslation('glossary');
  const country = node.nodeType === 'country' ? node : node.country;

  return (
    <div
      className={clsx(
        'ml-2 flex flex-row items-center gap-3 overflow-hidden',
        'cursor-default select-none',
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
          className={clsx('text-text-primary truncate text-base')}
          data-testid={`country-name-${country.code}`}
        >
          {name}
        </div>
        <div className="text-text-secondary text-sm">
          {`${gwCount} ${t('server', { count: gwCount })}`}
        </div>
      </div>
    </div>
  );
};

export default LocationInfo;
