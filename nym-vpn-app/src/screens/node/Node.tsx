import { useDeferredValue, useEffect, useMemo, useRef } from 'react';
import { useNavigate } from 'react-router';
import { Trans, useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { invoke } from '@tauri-apps/api/core';
import { useDialog } from '../../contexts';
import { NodeHop, isGateway } from '../../types';
import {
  SelectedUiNode,
  UiGateway,
  uiNodeToSelectedNode,
} from '../../types/node';
import { Link, PageAnim, TextInput } from '../../ui';
import { useI18nError } from '../../hooks';
import { useNodeListData } from '../../hooks/useNodeListData';
import { routes } from '../../router';
import { dispatch, useAppStore, useFetchGateways } from '../../store';
import { useNodeListState } from '../../store/nodeListState';
import { LocationDetailsDialog } from './location-details-dialog';
import { NodeList, useFilterList } from './list';

function Node({ node }: { node: NodeHop }) {
  const daemonStatus = useAppStore((s) => s.daemonStatus);
  const fetchGateways = useFetchGateways();

  const {
    loading,
    error,
    vpnMode,
    quicFilter,
    nodes: rawNodes,
    gateways: rawGateways,
  } = useNodeListData(node);

  const { isOpen, close } = useDialog();
  const {
    setFocused,
    exit: exitNodeList,
    entry: entryNodeList,
    reset: resetSaved,
    addToExpanded,
    setSearch,
  } = useNodeListState();

  const expanded =
    node === 'entry' ? entryNodeList.expanded : exitNodeList.expanded;
  const focused =
    node === 'entry' ? entryNodeList.focused : exitNodeList.focused;
  const search = node === 'entry' ? entryNodeList.search : exitNodeList.search;

  const { tE } = useI18nError();
  const navigate = useNavigate();
  const { t } = useTranslation('node-location');

  const { filter, nodes, gateways } = useFilterList(
    node,
    rawNodes,
    rawGateways,
    vpnMode,
  );
  const deferredNodes = useDeferredValue(nodes);
  const deferredGateways = useDeferredValue(gateways);
  const searchRef = useRef<HTMLInputElement>(null);

  const countriesCount = useMemo(() => {
    return new Set(nodes.map((node) => node.country.code)).size;
  }, [nodes]);

  const nodesCount = useMemo(() => {
    return nodes.reduce((acc, node) => acc + node.gateways.length, 0);
  }, [nodes]);

  useEffect(() => {
    if (daemonStatus === 'down') return;
    fetchGateways(vpnMode === 'mixnet' ? `mx-${node}` : 'wg');
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [node, vpnMode, daemonStatus]);

  useEffect(() => {
    if (searchRef.current) searchRef.current.focus({ preventScroll: true });
  }, []);

  const handleSelect = async (selected: SelectedUiNode) => {
    const selectedNode = uiNodeToSelectedNode(selected);
    if (
      isGateway(selectedNode) &&
      (selected.isSelected === 'exit' || selected.isSelected === 'entry')
    ) {
      return;
    }

    try {
      await invoke('set_node', {
        node: selectedNode,
        hop: node,
      });
      dispatch({
        type: 'set-node',
        payload: { hop: node, node: selectedNode },
      });
    } catch {
      /* TODO notify the user something went wrong */
    }
    navigate(routes.root);
    resetSaved(node);
  };

  const handleNodeDetails = (gateway: UiGateway) => {
    navigate(routes.nodeDetails, {
      state: { gateway, hop: node, resetScroll: true },
    });
    setFocused(node, { type: 'gateway', key: gateway.id });
    addToExpanded(node, gateway.country.code);
  };

  const onSearchChange = (value: string) => {
    setSearch(node, value);
    filter(value);
  };

  if (error) {
    return (
      <PageAnim
        className="flex h-full flex-col"
        data-testid="node-error-container"
      >
        <div
          className="h-2/3 w-4/5 overflow-auto text-center wrap-break-word"
          data-testid="node-error-message"
        >
          <p
            className="text-aphrodisiac font-medium"
            data-testid="node-error-title"
          >
            An error occurred
          </p>
          <p
            className="font-mono text-base"
            data-testid="node-error-details"
          >{`${tE(error.key)}: ${error.message} ${error.data?.details || '-'}`}</p>
        </div>
      </PageAnim>
    );
  }

  return (
    <>
      <LocationDetailsDialog
        isOpen={isOpen('location-info')}
        onClose={() => close('location-info')}
        node={node}
      />
      <PageAnim
        className="flex h-full flex-col"
        data-testid={`node-container-${node}`}
      >
        <div className="my-3 w-full px-6" data-testid="node-search-container">
          {quicFilter && (
            <p className="text-text-secondary mb-6 text-sm select-none">
              <Trans
                i18nKey="quic-filter-note"
                ns="node-location"
                components={{
                  here: (
                    <Link
                      text={t('here', { ns: 'glossary' })}
                      to={routes.antiCensorship}
                      className="text-black dark:text-white"
                      textClassName="underline-offset-2"
                    />
                  ),
                }}
              />
            </p>
          )}
          <TextInput
            ref={searchRef}
            onChange={onSearchChange}
            placeholder={t('search-country')}
            leftIcon="search"
            clearable
            value={search || ''}
          />
          <p className="text-text-secondary mt-3 text-sm">
            {t('countries-nodes', {
              countriesCount,
              nodesCount,
            })}
          </p>
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto">
          {loading && (
            <motion.div
              className="text-text-secondary mt-4 flex justify-center text-base"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.2, ease: 'easeOut' }}
              data-testid="node-loading-indicator"
            >
              {t('loading')}
            </motion.div>
          )}
          {!loading && (
            <NodeList
              nodes={deferredNodes}
              gateways={deferredGateways}
              onSelect={handleSelect}
              onNodeDetails={handleNodeDetails}
              hop={node}
              vpnMode={vpnMode}
              quicFilter={quicFilter}
              expanded={expanded}
              focused={focused}
            />
          )}
        </div>
      </PageAnim>
    </>
  );
}

export default Node;
