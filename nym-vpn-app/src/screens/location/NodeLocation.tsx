import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useDialog, useMainDispatch, useMainState } from '../../contexts';
import { kvSet } from '../../kvStore';
import { AppError, Country, NodeHop, StateDispatch } from '../../types';
import { routes } from '../../router';
import { useI18nError } from '../../hooks';
import { PageAnim, TextInput } from '../../ui';
import CountryList from './CountryList';
import LocationDetailsDialog from './LocationDetailsDialog';

export type UiCountry = {
  country: Country;
  i18n?: string;
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

  // the countries list used for UI rendering
  const [uiCountryList, setUiCountryList] = useState<UiCountry[]>([]);
  const selectedCountry =
    node === 'entry' ? entryNodeLocation : exitNodeLocation;

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

  // update the UI country list whenever the country list (likely from the backend)
  useEffect(() => {
    const countryList = node === 'entry' ? entryCountryList : exitCountryList;
    const list = [...countryList.map((country) => ({ country }))];
    setUiCountryList(list);
    setFilteredCountries(list);
    setSearch('');
  }, [node, entryCountryList, exitCountryList]);

  const filter = (value: string) => {
    if (value !== '') {
      const list = uiCountryList.filter((uiCountry) => {
        return uiCountry.country.name
          .toLowerCase()
          .startsWith(value.toLowerCase());
        // Use the toLowerCase() method to make it case-insensitive
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
        <div className="h-70 flex flex-col justify-center items-center gap-y-2 pt-3">
          <div className="w-full flex flex-row items-center px-4 mb-2">
            <TextInput
              value={search}
              onChange={filter}
              placeholder={t('search-country')}
              leftIcon="search"
              label={t('input-label')}
            />
          </div>
          <span className="mt-2" />
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
        </div>
      </PageAnim>
    </>
  );
}

export default NodeLocation;
