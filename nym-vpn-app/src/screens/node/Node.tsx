import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
// import { useNavigate } from 'react-router';
import {
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
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
  const { vpnMode } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  const { isOpen, close } = useDialog();
  const { nodes, loading, gateways } = useNodesState();

  const [uiNodes, setUiNodes] = useState<UiGatewaysByCountry[]>(nodes);
  const [uiGateways, setUiGateways] = useState<UiGateway[]>(gateways);
  const [search, setSearch] = useState('');

  // const navigate = useNavigate();
  const { t } = useTranslation('nodeLocation');

  // console.log(nodes);

  // refresh the UI list whenever the backend country data changes
  useEffect(() => {
    setUiNodes(nodes);
    setUiGateways([]);
    setSearch('');
  }, [nodes, gateways]);

  const filter = (value: string) => {
    if (value.length > 0) {
      const filteredNodes = nodes.filter((node) => {
        // toLowerCase() is used to make it case-insensitive
        return node.i18n.toLowerCase().includes(value.toLowerCase());
      });
      const filteredGw = gateways.filter((gw) => {
        return gw.name.toLowerCase().includes(value.toLowerCase());
      });
      console.log(`filteredGw ${filteredGw.length}`);
      setUiNodes(filteredNodes);
      setUiGateways(filteredGw);
    } else {
      setUiNodes(nodes);
      setUiGateways([]);
    }
    setSearch(value);
  };

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
            onChange={filter}
            placeholder={t('search-country')}
            leftIcon="search"
            label={t('input-label')}
          />
        </div>
        {loading && <div>loading...</div>}
        {!loading && (
          <NodeList
            nodes={uiNodes}
            gateways={uiGateways}
            onSelect={handleSelect}
            vpnMode={vpnMode}
          />
        )}
      </PageAnim>
    </>
  );
}

export default Node;
