import { useDeferredValue, useEffect, useMemo, useRef } from 'react';
import clsx from 'clsx';
import { useNavigate } from 'react-router';
import { Trans, useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '@headlessui/react';
import { useDialog } from '../../contexts';
import { NodeHop, isAuto, isGateway } from '../../types';
import {
  SelectedUiNode,
  UiGateway,
  uiNodeToSelectedNode,
} from '../../types/node';
import { Link, MsIcon, PageAnim, SmileyIcon, TextInput } from '../../ui';
import { useI18nError, useLang, useToast } from '../../hooks';
import { useNodeListData } from '../../hooks/useNodeListData';
import { routes } from '../../router';
import {
  dispatch,
  useAppStore,
  useFetchGateways,
  useFetchRecents,
} from '../../store';
import { useNodeListState } from '../../store/nodeListState';
import { useFavorites } from '../../store/favoritesState';
import { LocationDetailsDialog } from './location-details-dialog';
import {
  FavoritesEmpty,
  ListLoading,
  NodeList,
  RecentsPanel,
  ViewToggle,
  filterToFavorites,
  searchGateways,
  useFilterList,
} from './list';

const QUICK_PICK_CLASSES =
  'bg-surface-bg hover:bg-surface-hair flex cursor-default flex-row items-center gap-3 rounded-2xl p-4 transition-all duration-100';

function Node({ node }: { node: NodeHop }) {
  const daemonStatus = useAppStore((s) => s.daemonStatus);
  const storedNode = useAppStore((s) =>
    node === 'entry' ? s.entryNode : s.exitNode,
  );
  const fetchGateways = useFetchGateways();
  const fetchRecents = useFetchRecents();

  const {
    loading,
    recentsLoading,
    error,
    recentsError,
    vpnMode,
    quicFilter,
    nodes: rawNodes,
    gateways: rawGateways,
    recentGateways,
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
  const { getCountryName } = useLang();
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

  const searchedRecents = useMemo(
    () => searchGateways(recentGateways, search || '', getCountryName),
    [recentGateways, search, getCountryName],
  );

  const isRecents = view === 'recents';

  const countriesCount = useMemo(() => {
    if (isRecents) {
      return new Set(searchedRecents.map((gw) => gw.country.code)).size;
    }
    return new Set(nodes.map((node) => node.country.code)).size;
  }, [isRecents, searchedRecents, nodes]);

  const nodesCount = useMemo(() => {
    if (isRecents) return searchedRecents.length;
    return nodes.reduce((acc, node) => acc + node.gateways.length, 0);
  }, [isRecents, searchedRecents, nodes]);

  useEffect(() => {
    if (daemonStatus === 'down') return;
    fetchGateways(vpnMode === 'mixnet' ? `mx-${node}` : 'wg');
    // One call covers both hops — the daemon returns the entry and exit queues
    // together, keyed only on tunnel type.
    fetchRecents(vpnMode);
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
    } catch (error: unknown) {
      console.error('failed to set node', error);
      add({
        id: 'node-select-error',
        title: t('quick-pick.select-error'),
        type: 'error',
      });
      return;
    }
    navigate(routes.root);
    resetSaved(node);
  };

  const handleRandom = () => {
    handleSelect({ nodeType: 'random', isSelected: false });
  };

  const handleSafest = () => {
    handleSelect({ nodeType: 'safest', isSelected: false });
  };

  const randomActive = storedNode === 'random';
  // "Safest" is the daemon's `Auto` selection. Ignore the stored flag values —
  // a selection round-tripped from the daemon still highlights correctly
  // whatever it reports back.
  const safestActive = isAuto(storedNode);

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
          {isRecents ? (
            <RecentsPanel
              gateways={recentGateways}
              searched={searchedRecents}
              loading={recentsLoading}
              error={recentsError}
              onSelect={handleSelect}
              onNodeDetails={handleNodeDetails}
              hop={node}
              vpnMode={vpnMode}
              quicFilter={quicFilter}
            />
          ) : (
            <>
              {loading && <ListLoading />}
              {!loading && view === 'favorites' && favoritesEmpty && (
                <FavoritesEmpty hasFavorites={favorites.length > 0} />
              )}
              {!loading && !(view === 'favorites' && favoritesEmpty) && (
                <>
                  {view === 'all' && (
                    <div className="flex w-full flex-col gap-3 px-3 pt-3">
                      <Button
                        onClick={handleSafest}
                        className={clsx(QUICK_PICK_CLASSES, {
                          'border-brand-primary-active border-2': safestActive,
                        })}
                        data-testid="node-quick-pick-safest"
                      >
                        <SmileyIcon className="h-6 w-6" />
                        <span className="text-text-primary text-base">
                          {t('quick-pick.safest')}
                        </span>
                      </Button>
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
            </>
          )}
        </div>
      </PageAnim>
    </>
  );
}

export default Node;
