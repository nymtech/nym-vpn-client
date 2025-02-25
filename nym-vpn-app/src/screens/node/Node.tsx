import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import {
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
  useDialog,
  useMainDispatch,
  useMainState,
  useNodesState,
} from '../../contexts';
import { NodeHop, StateDispatch, isGateway } from '../../types';
import { PageAnim, TextInput } from '../../ui';
import { kvSet } from '../../kvStore';
import { uiNodeToRaw } from '../../contexts/nodes/util';
import { useI18nError } from '../../hooks';
import { routes } from '../../router';
import LocationDetailsDialog from './LocationDetailsDialog';
import { NodeList } from './list';
import NodeDetailsDialog from './NodeDetailsDialog';

function Node({ node }: { node: NodeHop }) {
  const { vpnMode } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  const { isOpen, close } = useDialog();
  const { nodes, loading, gateways, error } = useNodesState();
  const { tE } = useI18nError();
  const [nodeDetailsOpen, setNodeDetailsOpen] = useState(false);
  const nodeDetailsRef = useRef<UiGateway | UiCountry>(null);

  const [uiNodes, setUiNodes] = useState<UiGatewaysByCountry[]>(nodes);
  const [uiGateways, setUiGateways] = useState<UiGateway[]>(gateways);
  const [search, setSearch] = useState('');

  const navigate = useNavigate();
  const { t } = useTranslation('nodeLocation');

  // refresh the UI list whenever the backend gateway data changes
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
      setUiNodes(filteredNodes);
      setUiGateways(filteredGw);
    } else {
      setUiNodes(nodes);
      setUiGateways([]);
    }
    setSearch(value);
  };

  const handleSelect = async (selected: UiCountry | UiGateway) => {
    if (
      isGateway(selected) &&
      (selected.isSelected === 'exit' || selected.isSelected === 'entry')
    ) {
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
    navigate(routes.root);
  };

  const handleNodeDetails = (node: UiGateway | UiCountry) => {
    nodeDetailsRef.current = node;
    setNodeDetailsOpen(true);
  };

  if (error) {
    return (
      <PageAnim className="h-full flex flex-col">
        <div className="w-4/5 h-2/3 overflow-auto break-words text-center">
          <p className="text-teaberry font-semibold">An error occurred</p>
          <p className="text-base font-mono">{`${tE(error.key)}: ${error.message} ${error.data?.details || '-'}`}</p>
        </div>
      </PageAnim>
    );
  }

  return (
    <>
      <NodeDetailsDialog
        isOpen={nodeDetailsOpen}
        onClose={() => setNodeDetailsOpen(false)}
        ref={nodeDetailsRef}
      />
      <LocationDetailsDialog
        isOpen={isOpen('location-info')}
        onClose={() => close('location-info')}
      />
      <PageAnim className="h-full flex flex-col">
        <div className="w-full max-w-md px-6 mt-6 mb-6">
          <TextInput
            value={search}
            onChange={filter}
            placeholder={t('search-country')}
            leftIcon="search"
            label={t('input-label')}
          />
        </div>
        {loading && <div>{t('loading')}</div>}
        {!loading && (
          <NodeList
            nodes={uiNodes}
            gateways={uiGateways}
            onSelect={handleSelect}
            onNodeDetails={handleNodeDetails}
            node={node}
            vpnMode={vpnMode}
          />
        )}
      </PageAnim>
    </>
  );
}

export default Node;
