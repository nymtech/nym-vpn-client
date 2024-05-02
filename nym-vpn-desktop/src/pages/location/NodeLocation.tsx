import { useTranslation } from 'react-i18next';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api';
import { useMainDispatch, useMainState } from '../../contexts';
import {
  CmdError,
  Country,
  NodeHop,
  StateDispatch,
  NodeLocation as TNodeLocation,
  isCountry,
} from '../../types';
import { FastestFeatureEnabled } from '../../constants';
import { routes } from '../../router';
import { PageAnim, TextInput } from '../../ui';
import CountryList from './CountryList';

export type UiCountry = {
  country: Country;
  isFastest: boolean;
};

function NodeLocation({ node }: { node: NodeHop }) {
  const { t } = useTranslation('nodeLocation');
  const {
    entryNodeLocation,
    exitNodeLocation,
    entryCountryList,
    exitCountryList,
    fastestNodeLocation,
    entryCountriesLoading,
    exitCountriesLoading,
    fetchEntryCountries,
    fetchExitCountries,
  } = useMainState();

  // the countries list used for UI rendering, Fastest country is at first position
  const [uiCountryList, setUiCountryList] = useState<UiCountry[]>(
    FastestFeatureEnabled
      ? [{ country: fastestNodeLocation, isFastest: true }]
      : [],
  );

  const [search, setSearch] = useState('');
  const [filteredCountries, setFilteredCountries] =
    useState<UiCountry[]>(uiCountryList);

  const dispatch = useMainDispatch() as StateDispatch;

  const navigate = useNavigate();

  // request backend to refresh cache
  useEffect(() => {
    if (node === 'entry') {
      fetchEntryCountries();
    } else {
      fetchExitCountries();
    }
    if (FastestFeatureEnabled) {
      invoke<Country>('get_fastest_node_location')
        .then((country) => {
          dispatch({ type: 'set-fastest-node-location', country });
        })
        .catch((e: CmdError) => console.error(e));
    }
  }, [node, dispatch, fetchEntryCountries, fetchExitCountries]);

  // update the UI country list whenever the country list or
  // fastest country change (likely from the backend)
  useEffect(() => {
    const countryList = node === 'entry' ? entryCountryList : exitCountryList;
    const list = [
      ...countryList.map((country) => ({ country, isFastest: false })),
    ];
    if (FastestFeatureEnabled) {
      // put fastest country at the first position
      list.unshift({ country: fastestNodeLocation, isFastest: true });
    }
    setUiCountryList(list);
    setFilteredCountries(list);
    setSearch('');
  }, [node, entryCountryList, exitCountryList, fastestNodeLocation]);

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

  const isCountrySelected = (
    selectedNode: TNodeLocation,
    country: UiCountry,
  ): boolean => {
    if (selectedNode === 'Fastest' && country.isFastest) {
      return true;
    }
    return (
      selectedNode !== 'Fastest' && selectedNode.code === country.country.code
    );
  };

  const handleCountrySelection = async (country: UiCountry) => {
    const location = country.isFastest ? 'Fastest' : country.country;

    try {
      await invoke<void>('set_node_location', {
        nodeType: node === 'entry' ? 'Entry' : 'Exit',
        location: isCountry(location) ? { Country: location } : 'Fastest',
      });
      dispatch({
        type: 'set-node-location',
        payload: { hop: node, location },
      });
    } catch (e) {
      console.warn(e);
    }
    navigate(routes.root);
  };

  return (
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
        <CountryList
          countries={filteredCountries}
          loading={
            node === 'entry' ? entryCountriesLoading : exitCountriesLoading
          }
          onSelect={handleCountrySelection}
          isSelected={(country: UiCountry) => {
            return isCountrySelected(
              node === 'entry' ? entryNodeLocation : exitNodeLocation,
              country,
            );
          }}
        />
      </div>
    </PageAnim>
  );
}

export default NodeLocation;
