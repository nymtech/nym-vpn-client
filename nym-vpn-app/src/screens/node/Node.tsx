import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  useDialog,
  useMainDispatch,
  useMainState,
  useNodesState,
} from '../../contexts';
import { Country, Gateway, NodeHop, StateDispatch } from '../../types';
import { PageAnim, TextInput } from '../../ui';
import { kvSet } from '../../kvStore';
import LocationDetailsDialog from './LocationDetailsDialog';

let initialized = false;

function Node({ node }: { node: NodeHop }) {
  const { vpnMode, fetchGateways } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  const { isOpen, close } = useDialog();
  const { nodes, loading } = useNodesState();

  const [search, setSearch] = useState('');

  const { t } = useTranslation('nodeLocation');

  // console.log(nodes);

  // refresh cache (if stale)
  useEffect(() => {
    if (initialized) {
      return;
    }
    initialized = true;
    if (vpnMode === 'Mixnet') {
      fetchGateways(`mx-${node}`);
    } else {
      fetchGateways('wg');
    }
  }, [node, vpnMode, fetchGateways]);

  // const dispatch = useMainDispatch() as StateDispatch;
  // const navigate = useNavigate();

  // const filter = (value: string) => {
  //   if (value !== '') {
  //     const list = uiCountryList.filter((uiCountry) => {
  //       // toLowerCase() is used to make it case-insensitive
  //       return uiCountry.i18n.toLowerCase().includes(value.toLowerCase());
  //     });
  //     setFilteredCountries(list);
  //   } else {
  //     setFilteredCountries(uiCountryList);
  //   }
  //   setSearch(value);
  // };

  const handleSelection = async (selected: Country | Gateway) => {
    // TODO cancel if the selected node is already assigned
    // to the other hop
    try {
      await kvSet(node === 'entry' ? 'entry-node' : 'exit-node', selected);
      dispatch({
        type: 'set-node',
        payload: { hop: node, node: selected },
      });
    } catch (e) {
      console.warn(e);
    }
    // navigate(routes.root);
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
            onChange={() => {
              /* TODO call filter fn */
            }}
            placeholder={t('search-country')}
            leftIcon="search"
            label={t('input-label')}
          />
        </div>
        {loading && <div>loading...</div>}
        {!loading &&
          nodes.map(({ i18n, country, gateways, isSelected }) => (
            <div key={country.code} onClick={() => handleSelection(country)}>
              <div className="text-2xl font-bold text-malachite">{`[${country.code}] ${i18n} selected: ${isSelected}`}</div>
              {gateways.map((gateway) => (
                <div
                  className="ml-4 text-liquid-lava"
                  key={gateway.id}
                  onClick={() => handleSelection(gateway)}
                >
                  <span className="text-cornflower font-mono">
                    {gateway.id.slice(0, 6)}
                  </span>
                  <span>{` ${gateway.name.slice(0, 30)}`}</span>
                  <span className="text-cornflower font-mono">
                    {vpnMode === 'Mixnet'
                      ? ` ${gateway.mxScore}`
                      : ` ${gateway.wgScore}`}
                  </span>
                </div>
              ))}
            </div>
          ))}
      </PageAnim>
    </>
  );
}

export default Node;
