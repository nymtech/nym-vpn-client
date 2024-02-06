import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { UiCountry } from './NodeLocation';

interface CountryListProps {
  countries: UiCountry[];
  onSelect: (country: UiCountry) => void;
  isSelected: (country: UiCountry) => boolean;
}

export default function CountryList({
  countries,
  onSelect,
  isSelected,
}: CountryListProps) {
  const { t } = useTranslation('nodeLocation');

  return (
    <ul className="flex flex-col w-full items-stretch gap-1">
      {countries && countries.length > 0 ? (
        countries.map((uiCountry) => (
          <li
            key={uiCountry.isFastest ? 'fastest' : uiCountry.country.code}
            className="list-none w-full"
          >
            <div
              role="presentation"
              onKeyDown={() => onSelect(uiCountry)}
              className={clsx([
                'flex flex-row justify-between',
                'hover:bg-gun-powder hover:bg-opacity-10',
                'dark:hover:bg-laughing-jack dark:hover:bg-opacity-10',
                'rounded-lg cursor-pointer px-3 py-1',
              ])}
              onClick={() => onSelect(uiCountry)}
            >
              {!uiCountry.isFastest && (
                <div className="flex flex-row items-center m-1 gap-3 p-1 cursor-pointer">
                  <div className="w-7 flex justify-center items-center">
                    <img
                      src={`./flags/${uiCountry.country.code.toLowerCase()}.svg`}
                      className="h-6"
                      alt={uiCountry.country.code}
                    />
                  </div>
                  <div className="flex items-center dark:text-mercury-pinkish text-base cursor-pointer">
                    {uiCountry.country.name}
                  </div>
                </div>
              )}
              {uiCountry.isFastest && (
                <div className="flex flex-row items-center m-1 gap-3 p-1 cursor-pointer">
                  <div className="w-7 max-h-6 flex justify-center items-center">
                    <span className="font-icon text-2xl cursor-pointer">
                      bolt
                    </span>
                  </div>
                  <div className="cursor-pointer text-base">{`${t('fastest', {
                    ns: 'common',
                  })} (${uiCountry.country.name})`}</div>
                </div>
              )}
              <div
                className={clsx([
                  'pr-4 flex items-center font-medium text-xs cursor-pointer',
                  'text-cement-feet dark:text-mercury-mist',
                ])}
              >
                {isSelected(uiCountry) && t('selected')}
              </div>
            </div>
          </li>
        ))
      ) : (
        <p className="flex justify-center">{t('none-found')}</p>
      )}
    </ul>
  );
}
