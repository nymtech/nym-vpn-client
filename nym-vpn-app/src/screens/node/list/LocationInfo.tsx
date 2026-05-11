import clsx from 'clsx';
import { SelectedKind, UiCountry, UiRegion } from '../../../types/node';
import { FlagIcon, countryCode } from '../../../ui';

type LocationInfoProps = {
  node: UiCountry | UiRegion;
  name: string;
  isSelected: SelectedKind;
  hop: 'entry' | 'exit';
  hideFlag?: boolean;
};

const LocationInfo = ({
  node,
  name,
  isSelected,
  hop,
  hideFlag,
}: LocationInfoProps) => {
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
          className={clsx(
            'box-content size-8! min-h-8! min-w-8! rounded-full',
            isSelected && 'border-2',
            (isSelected === hop || isSelected === 'entry-and-exit') &&
              'border-primary-active',
            isSelected && isSelected !== hop && 'border-text-secondary',
          )}
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
