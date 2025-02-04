import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { FlagIcon, countryCode } from '../../ui';
import { UiCountry } from './NodeLocation';

type CountryListProps = {
  countries: UiCountry[];
  onSelect: (country: UiCountry) => void;
  isSelected: (country: UiCountry) => boolean;
  loading: boolean;
};

export default function CountryList({
  countries = [],
  onSelect,
  isSelected,
  loading,
}: CountryListProps) {
  const { t } = useTranslation('nodeLocation');

  if (loading && countries.length === 0) {
    return (
      <p className="flex justify-center dark:text-mercury-pinkish">
        {t('list-loading')}
      </p>
    );
  }

  return (
    <ul className="flex flex-col w-full items-stretch gap-1">
      {countries.length > 0 ? (
        countries.map((uiCountry) => (
          <li key={uiCountry.country.code} className="list-none w-full">
            <div
              role="presentation"
              onKeyDown={() => onSelect(uiCountry)}
              className={clsx([
                'flex flex-row justify-between',
                'hover:bg-gun-powder hover:bg-opacity-10',
                'dark:hover:bg-laughing-jack dark:hover:bg-opacity-10',
                'rounded-lg px-3 py-1 transition duration-75 cursor-default',
              ])}
              onClick={() => onSelect(uiCountry)}
            >
              <div className="flex flex-row items-center m-1 gap-3 p-1 overflow-hidden">
                <FlagIcon
                  code={uiCountry.country.code.toLowerCase() as countryCode}
                  alt={uiCountry.country.code}
                  className="h-6"
                />
                <div className="dark:text-mercury-pinkish text-base truncate">
                  {uiCountry.country.name}
                </div>
              </div>
              <div
                className={clsx([
                  'pr-4 ml-2 flex items-center font-medium text-xs',
                  'text-cement-feet dark:text-mercury-mist',
                ])}
              >
                {isSelected(uiCountry) && t('selected')}
              </div>
            </div>
          </li>
        ))
      ) : (
        <p className="flex justify-center dark:text-mercury-pinkish">
          {t('none-found')}
        </p>
      )}
    </ul>
  );
}
