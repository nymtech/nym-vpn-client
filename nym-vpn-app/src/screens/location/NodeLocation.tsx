import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useDialog, useMainDispatch, useMainState } from '../../contexts';
import { kvSet } from '../../kvStore';
import { AppError, Country, NodeHop, StateDispatch } from '../../types';
import { routes } from '../../router';
import { useI18nError, useLang } from '../../hooks';
import { PageAnim, TextInput } from '../../ui';
import CountryList from './CountryList';
import LocationDetailsDialog from './LocationDetailsDialog';

// Thin wrapper around `Country` that includes localization
export type UiCountry = {
  country: Country;
  i18n: string;
};

function NodeLocation({ node }: { node: NodeHop }) {
  const {
    entryNodeLocation,
    exitNodeLocation,
    entryCountryList,
    exitCountryList,
    entryCountriesLoading,
    exitCountriesLoading,
    fetchMnCountries,
    fetchWgCountries,
    entryCountriesError,
    exitCountriesError,
    vpnMode,
  } = useMainState();
  const { isOpen, close } = useDialog();

  const { t } = useTranslation('nodeLocation');
  const { tE } = useI18nError();
  const { compare, getCountryName } = useLang();

  // the country list as rendered in the UI
  const [uiCountryList, setUiCountryList] = useState<UiCountry[]>([]);
  const selectedCountry =
    node === 'entry' ? entryNodeLocation : exitNodeLocation;
  const countryList = node === 'entry' ? entryCountryList : exitCountryList;

  const [search, setSearch] = useState('');
  const [filteredCountries, setFilteredCountries] =
    useState<UiCountry[]>(uiCountryList);

  const dispatch = useMainDispatch() as StateDispatch;
  const navigate = useNavigate();

  // refresh cache (if stale)
  useEffect(() => {
    if (vpnMode === 'Mixnet') {
      fetchMnCountries(node);
    } else {
      fetchWgCountries();
    }
  }, [node, vpnMode, fetchWgCountries, fetchMnCountries]);

  // refresh the UI country list whenever the backend country data changes
  useEffect(() => {
    const uiList = countryList
      .map((country) => {
        return {
          country,
          i18n: getCountryName(country.code) || country.name,
        };
      })
      .sort((a, b) => compare(a.i18n, b.i18n));
    setUiCountryList(uiList);
    setFilteredCountries(uiList);
    setSearch('');
  }, [countryList, compare, getCountryName]);

  const filter = (value: string) => {
    if (value !== '') {
      const list = uiCountryList.filter((uiCountry) => {
        // toLowerCase() is used to make it case-insensitive
        return uiCountry.i18n.toLowerCase().includes(value.toLowerCase());
      });
      setFilteredCountries(list);
    } else {
      setFilteredCountries(uiCountryList);
    }
    setSearch(value);
  };

  const handleCountrySelection = async (country: UiCountry) => {
    const location = country.country;

    try {
      await kvSet<Country>(
        node === 'entry' ? 'EntryNodeLocation' : 'ExitNodeLocation',
        location,
      );
      dispatch({
        type: 'set-node-location',
        payload: { hop: node, location },
      });
    } catch (e) {
      console.warn(e);
    }
    navigate(routes.root);
  };

  const error =
    (node === 'entry' && entryCountriesError) ||
    (node === 'exit' && exitCountriesError);

  const renderError = (e: AppError) => (
    <div className="w-4/5 h-2/3 overflow-auto break-words text-center">
      <p className="text-sm text-teaberry font-bold">{`${tE(e.key)}: ${e.message} ${e.data?.details || '-'}`}</p>
    </div>
  );

  return (
    <>
      <LocationDetailsDialog
        isOpen={isOpen('location-info')}
        onClose={() => close('location-info')}
      />
      <PageAnim className="h-full flex flex-col">
        <div className="w-full max-w-md px-4 mt-4 mb-6">
          <TextInput
            value={search}
            onChange={filter}
            placeholder={t('search-country')}
            leftIcon="search"
            label={t('input-label')}
          />
        </div>
        {error ? (
          renderError(error)
        ) : (
          <CountryList
            countries={filteredCountries}
            loading={
              node === 'entry' ? entryCountriesLoading : exitCountriesLoading
            }
            onSelect={(country) => {
              handleCountrySelection(country);
            }}
            isSelected={(country: UiCountry) =>
              selectedCountry.code === country.country.code
            }
          />
        )}
      </PageAnim>
    </>
  );
}

export default NodeLocation;
