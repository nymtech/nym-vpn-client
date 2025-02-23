import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
// import { useNavigate } from 'react-router';
import {
  UiCountry,
  UiGateway,
  useDialog,
  useMainDispatch,
  useMainState,
  useNodesState,
} from '../../contexts';
import { NodeHop, StateDispatch, isCountry } from '../../types';
import { PageAnim, TextInput } from '../../ui';
import { kvSet } from '../../kvStore';
import { uiNodeToRaw } from '../../contexts/nodes/util';
import LocationDetailsDialog from './LocationDetailsDialog';
import { NodeList } from './list';

function Node({ node }: { node: NodeHop }) {
  const { vpnMode, fetchGateways } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  const { isOpen, close } = useDialog();
  const { nodes, loading } = useNodesState();

  const [search, setSearch] = useState('');

  // const navigate = useNavigate();
  const { t } = useTranslation('nodeLocation');

  // console.log(nodes);

  // refresh cache (if stale)
  useEffect(() => {
    if (vpnMode === 'Mixnet') {
      fetchGateways(`mx-${node}`);
    } else {
      fetchGateways('wg');
    }
  }, [node, vpnMode, fetchGateways]);

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

  const handleSelect = async (selected: UiCountry | UiGateway) => {
    if (selected.isSelected === 'exit' || selected.isSelected === 'entry') {
      // TODO remove this log
      console.log(
        `${isCountry(selected) ? 'country' : 'gateway'} already selected by ${selected.isSelected} node`,
      );
      return;
    }

    try {
      await kvSet(
        node === 'entry' ? 'entry-node' : 'exit-node',
        uiNodeToRaw(selected),
      );
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
        {!loading && (
          <NodeList nodes={nodes} onSelect={handleSelect} vpnMode={vpnMode} />
        )}
      </PageAnim>
    </>
  );
}

export default Node;
