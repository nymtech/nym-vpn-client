import { useTranslation } from 'react-i18next';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api';
import { useMainDispatch, useMainState } from '../../contexts';
import {
  CmdError,
  Country,
  InputEvent,
  NodeHop,
  NodeLocation,
  StateDispatch,
  isCountry,
} from '../../types';
import { FastestFeatureEnabled, routes } from '../../constants';
import SearchBox from './SearchBox';
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
    countryList,
    fastestNodeLocation,
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
    invoke<Country[]>('get_node_countries')
      .then((countries) => {
        dispatch({
          type: 'set-country-list',
          countries,
        });
      })
      .catch((e: CmdError) => console.error(e));
    if (FastestFeatureEnabled) {
      invoke<Country>('get_fastest_node_location')
        .then((country) => {
          dispatch({ type: 'set-fastest-node-location', country });
        })
        .catch((e: CmdError) => console.error(e));
    }
  }, [dispatch]);

  // update the UI country list whenever the country list or
  // fastest country change (likely from the backend)
  useEffect(() => {
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
  }, [countryList, fastestNodeLocation]);

  const filter = (e: InputEvent) => {
    const keyword = e.target.value;
    if (keyword !== '') {
      const list = uiCountryList.filter((uiCountry) => {
        return uiCountry.country.name
          .toLowerCase()
          .startsWith(keyword.toLowerCase());
        // Use the toLowerCase() method to make it case-insensitive
      });
      setFilteredCountries(list);
    } else {
      setFilteredCountries(uiCountryList);
    }
    setSearch(keyword);
  };

  const isCountrySelected = (
    selectedNode: NodeLocation,
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
    console.log(country);
    const location: NodeLocation = country.isFastest
      ? 'Fastest'
      : country.country;

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
      console.log(e);
    }
    navigate(routes.root);
  };

  return (
    <div className="h-full flex flex-col">
      <div className="h-70 flex flex-col justify-center items-center gap-y-2 pt-3">
        <SearchBox
          value={search}
          onChange={filter}
          placeholder={t('search-country')}
        />
        <span className="mt-2" />
        <CountryList
          countries={filteredCountries}
          onSelect={handleCountrySelection}
          isSelected={(country: UiCountry) => {
            return isCountrySelected(
              node === 'entry' ? entryNodeLocation : exitNodeLocation,
              country,
            );
          }}
        />
      </div>
    </div>
  );
}

export default NodeLocation;
