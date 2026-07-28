import { useDeferredValue, useEffect, useMemo, useRef } from 'react';
import clsx from 'clsx';
import { useNavigate } from 'react-router';
import { Trans, useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@headlessui/react';
import { useDialog } from '../../contexts';
import { GatewaySelectionAlgorithm, NodeHop, isGateway } from '../../types';
import {
  SelectedUiNode,
  UiGateway,
  uiNodeToSelectedNode,
} from '../../types/node';
import { Link, MsIcon, PageAnim, /* SmileyIcon, */ TextInput } from '../../ui';
import { useI18nError, useToast } from '../../hooks';
import { useNodeListData } from '../../hooks/useNodeListData';
import { routes } from '../../router';
import { dispatch, useAppStore, useFetchGateways } from '../../store';
import { useNodeListState } from '../../store/nodeListState';
import { useFavorites } from '../../store/favoritesState';
import { LocationDetailsDialog } from './location-details-dialog';
import {
  FavoritesEmpty,
  NodeList,
  ViewToggle,
  filterToFavorites,
  useFilterList,
} from './list';

const QUICK_PICK_CLASSES =
  'bg-surface-bg hover:bg-surface-hair flex cursor-default flex-row items-center gap-3 rounded-2xl p-4 transition-all duration-100';

function Node({ node }: { node: NodeHop }) {
  const daemonStatus = useAppStore((s) => s.daemonStatus);
  const algoConfig = useAppStore((s) => s.gatewaySelectionAlgorithmConfig);
  const storedNode = useAppStore((s) =>
    node === 'entry' ? s.entryNode : s.exitNode,
  );
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
    setView,
  } = useNodeListState();

  const expanded =
    node === 'entry' ? entryNodeList.expanded : exitNodeList.expanded;
  const focused =
    node === 'entry' ? entryNodeList.focused : exitNodeList.focused;
  const search = node === 'entry' ? entryNodeList.search : exitNodeList.search;
  const view = node === 'entry' ? entryNodeList.view : exitNodeList.view;
  const favorites = useFavorites(node);

  const { tE } = useI18nError();
  const navigate = useNavigate();
  const { add } = useToast();
  const { t } = useTranslation('node-location');

  const viewNodes = useMemo(
    () => (view === 'favorites' ? filterToFavorites(rawNodes) : rawNodes),
    [view, rawNodes],
  );
  const viewGateways = useMemo(() => {
    if (view !== 'favorites') return rawGateways;
    const flat: UiGateway[] = [];
    for (const country of viewNodes) flat.push(...country.gateways);
    const visible = new Set(flat.map((gw) => gw.id));
    return rawGateways.filter((gw) => visible.has(gw.id));
  }, [view, rawGateways, viewNodes]);

  const favoritesEmpty = viewNodes.length === 0 && viewGateways.length === 0;

  const { filter, nodes, gateways } = useFilterList(
    node,
    viewNodes,
    viewGateways,
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

  const rollbackAlgo = async (algorithm: GatewaySelectionAlgorithm) => {
    try {
      await invoke('set_gateway_selection_algorithm', { algorithm });
      dispatch({
        type: 'set-gateway-selection-algorithm-config',
        config: { ...algoConfig, gatewaySelectionAlgorithm: algorithm },
      });
    } catch (rollbackError: unknown) {
      console.error(
        `failed to rollback gateway selection algorithm to [${algorithm}]`,
        rollbackError,
      );
    }
  };

  const handleSelect = async (selected: SelectedUiNode) => {
    const selectedNode = uiNodeToSelectedNode(selected);
    if (
      isGateway(selectedNode) &&
      (selected.isSelected === 'exit' || selected.isSelected === 'entry')
    ) {
      return;
    }

    // Picking an exit while in 'auto' (daemon picks both) means the user is
    // now explicit about the exit — flip to 'autoEntryExplicitExit'. The
    // entry hop stays daemon-picked. The mirror flip 'non-explicit → explicit'
    // on entry-pick is gone: the entry list is only reachable from
    // 'explicit', where that flip would be a no-op.
    // Algorithm change is applied first so a failure aborts before the node
    // state diverges from the daemon. If the node update later fails, the
    // algorithm change is rolled back.
    const needsAlgoFlip =
      node === 'exit' && algoConfig.gatewaySelectionAlgorithm === 'auto';
    if (needsAlgoFlip) {
      try {
        await invoke('set_gateway_selection_algorithm', {
          algorithm: 'autoEntryExplicitExit',
        });
        dispatch({
          type: 'set-gateway-selection-algorithm-config',
          config: {
            ...algoConfig,
            gatewaySelectionAlgorithm: 'autoEntryExplicitExit',
          },
        });
      } catch (error: unknown) {
        console.error(
          'failed to set gateway selection algorithm to [autoEntryExplicitExit]',
          error,
        );
        add({
          id: 'node-select-error',
          title: t('quick-pick.select-error'),
          type: 'error',
        });
        return;
      }
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
    } catch (error: unknown) {
      console.error('failed to set node', error);
      add({
        id: 'node-select-error',
        title: t('quick-pick.select-error'),
        type: 'error',
      });
      if (needsAlgoFlip) {
        await rollbackAlgo('auto');
      }
      return;
    }
    navigate(routes.root);
    resetSaved(node);
  };

  const handleRandom = () => {
    handleSelect({ nodeType: 'random', isSelected: false });
  };

  /*
  const handleBestServer = async () => {
    // Switch the algorithm to 'auto' first so a failure aborts before the
    // stored node is cleared. If the node clear later fails, the algorithm
    // change is rolled back to the previous value.
    const previousAlgo = algoConfig.gatewaySelectionAlgorithm;
    const needsAlgoFlip = previousAlgo !== 'auto';
    if (needsAlgoFlip) {
      try {
        await invoke('set_gateway_selection_algorithm', { algorithm: 'auto' });
        dispatch({
          type: 'set-gateway-selection-algorithm-config',
          config: {
            ...algoConfig,
            gatewaySelectionAlgorithm: 'auto',
          },
        });
      } catch (error: unknown) {
        console.error(
          'failed to set gateway selection algorithm to [auto]',
          error,
        );
        add({
          id: 'node-select-error',
          title: t('quick-pick.select-error'),
          type: 'error',
        });
        return;
      }
    }

    // Clear the stored exit pick so it isn't re-applied next time the user
    // visits Auto (ModeToggle derives algo from exitNode now).
    try {
      await invoke('set_node', { node: 'random', hop: node });
      dispatch({
        type: 'set-node',
        payload: { hop: node, node: 'random' },
      });
    } catch (error: unknown) {
      console.error('failed to clear exit node selection', error);
      add({
        id: 'node-select-error',
        title: t('quick-pick.select-error'),
        type: 'error',
      });
      if (needsAlgoFlip) {
        await rollbackAlgo(previousAlgo);
      }
      return;
    }
    navigate(routes.root);
    resetSaved(node);
  };

  const showBestServer =
    node === 'exit' &&
    (algoConfig.gatewaySelectionAlgorithm === 'auto' ||
      algoConfig.gatewaySelectionAlgorithm === 'autoEntryExplicitExit');

  const bestServerActive =
    node === 'exit' && algoConfig.gatewaySelectionAlgorithm === 'auto';
  */

  // Random is only "active" when the user actually owns this hop's selection;
  // in daemon-picked algos the stored 'random' is treated as no selection
  // (mirrors useNodeListData).
  const randomActive =
    storedNode === 'random' &&
    (node === 'entry'
      ? algoConfig.gatewaySelectionAlgorithm === 'explicit'
      : algoConfig.gatewaySelectionAlgorithm !== 'auto');

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
            className="text-status-error font-medium"
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
          <div className="mt-3">
            <ViewToggle view={view} onChange={(v) => setView(node, v)} />
          </div>
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
          {!loading && view === 'favorites' && favoritesEmpty && (
            <FavoritesEmpty hasFavorites={favorites.length > 0} />
          )}
          {!loading && !(view === 'favorites' && favoritesEmpty) && (
            <>
              {view === 'all' && (
                <div className="flex w-full flex-col gap-3 px-3 pt-3">
                  <Button
                    onClick={handleRandom}
                    className={clsx(QUICK_PICK_CLASSES, {
                      'border-brand-primary-active border-2': randomActive,
                    })}
                  >
                    <MsIcon icon="shuffle" className="text-text-primary" />
                    <span className="text-text-primary text-base">
                      {t('quick-pick.random')}
                    </span>
                  </Button>
                  {/* {showBestServer && (
                    <Button
                      onClick={handleBestServer}
                      className={clsx(QUICK_PICK_CLASSES, {
                        'border-brand-primary-active border-2': bestServerActive,
                      })}
                    >
                      <SmileyIcon className="h-6 w-6" />
                      <span className="text-text-primary text-base">
                        {t('quick-pick.best-server')}
                      </span>
                    </Button>
                  )} */}
                </div>
              )}
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
            </>
          )}
        </div>
      </PageAnim>
    </>
  );
}

export default Node;
