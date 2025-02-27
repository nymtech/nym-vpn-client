import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
  useDialog,
  useNodesState,
} from '../../contexts';
import { NodeHop } from '../../types';
import { PageAnim, TextInput } from '../../ui';
import { useI18nError } from '../../hooks';
import LocationDetailsDialog from './LocationDetailsDialog';
import { NodeList } from './list';
import NodeDetailsDialog from './NodeDetailsDialog';

function Node({ node }: { node: NodeHop }) {
  const { nodes, loading, gateways, error, vpnMode, onNodeSelect } =
    useNodesState();

  const { isOpen, close } = useDialog();
  const { tE } = useI18nError();
  const [nodeDetailsOpen, setNodeDetailsOpen] = useState(false);
  const nodeDetailsRef = useRef<UiGateway | UiCountry>(null);

  const [uiNodes, setUiNodes] = useState<UiGatewaysByCountry[]>(nodes);
  const [uiGateways, setUiGateways] = useState<UiGateway[]>(gateways);
  const [search, setSearch] = useState('');

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

  const handleSelect = async (selected: UiGateway | UiCountry) => {
    return onNodeSelect(node, selected);
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
