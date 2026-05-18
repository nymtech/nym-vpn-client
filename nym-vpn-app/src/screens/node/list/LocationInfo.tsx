import clsx from 'clsx';
import { UiCountry, UiRegion } from '../../../types/node';
import { FlagIcon, countryCode } from '../../../ui';

type LocationInfoProps = {
  node: UiCountry | UiRegion;
  name: string;
  hideFlag?: boolean;
};

const LocationInfo = ({ node, name, hideFlag }: LocationInfoProps) => {
  const country = node.nodeType === 'country' ? node : node.country;

  return (
    <div
      className="flex cursor-default items-center gap-4 overflow-hidden pl-4 select-none"
      data-testid={`country-info-${country.code}`}
    >
      {!hideFlag && (
        <FlagIcon
          code={country.code.toLowerCase() as countryCode}
          alt={country.code}
          className={clsx('box-content size-8! min-h-8! min-w-8! rounded-full')}
          data-testid={`country-flag-${country.code}`}
        />
      )}
      <div className="flex flex-col justify-center overflow-hidden pr-4">
        <div
          className="text-text-primary truncate text-lg"
          data-testid={`country-name-${country.code}`}
        >
          {name}
        </div>
      </div>
    </div>
  );
};

export default LocationInfo;
