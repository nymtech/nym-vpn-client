import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useDialog, useMainDispatch, useMainState } from '../../contexts';
import { kvSet } from '../../kvStore';
import { AppError, Country, NodeHop, StateDispatch } from '../../types';
import { routes } from '../../router';
import { useI18nError, useLang } from '../../hooks';
import { PageAnim, TextInput } from '../../ui';
import LocationDetailsDialog from './LocationDetailsDialog';

// Thin wrapper around `Country` that includes localization
export type UiCountry = {
  country: Country;
  i18n: string;
};

function Node({ node }: { node: NodeHop }) {
  const { entryNode, exitNode, vpnMode, fetchMxGateways, fetchWgGateways } =
    useMainState();
  const { isOpen, close } = useDialog();

  const { t } = useTranslation('nodeLocation');
  const { tE } = useI18nError();
  const { compare, getCountryName } = useLang();

  // the country list as rendered in the UI
  const [uiCountryList, setUiCountryList] = useState<UiCountry[]>([]);
  const selectedCountry = node === 'entry' ? entryNode : exitNode;

  const [search, setSearch] = useState('');
  const [filteredCountries, setFilteredCountries] =
    useState<UiCountry[]>(uiCountryList);

  // const dispatch = useMainDispatch() as StateDispatch;
  // const navigate = useNavigate();

  // refresh cache (if stale)
  useEffect(() => {
    if (vpnMode === 'Mixnet') {
      fetchMxGateways(node);
    } else {
      fetchWgGateways();
    }
  }, [node, vpnMode, fetchMxGateways, fetchWgGateways]);

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
        TODO
      </PageAnim>
    </>
  );
}

export default Node;
