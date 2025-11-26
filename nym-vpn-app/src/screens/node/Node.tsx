import { useDeferredValue, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router';
import { Trans, useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import {
  SelectedUiNode,
  UiGateway,
  useDialog,
  useMainDispatch,
  useMainState,
  useNodeList,
  useNodeListState,
} from '../../contexts';
import { NodeHop, StateDispatch } from '../../types';
import { Link, PageAnim, TextInput } from '../../ui';
import { kvSet } from '../../kvStore';
import { uiNodeToSelectedNode } from '../../contexts/node-list/util';
import { useI18nError } from '../../hooks';
import { routes } from '../../router';
import LocationDetailsDialog from './LocationDetailsDialog';
import { NodeList, useFilterList } from './list';

function Node({ node }: { node: NodeHop }) {
  const { backendFlags, vpnMode, quic } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;

  const { isOpen, close } = useDialog();
  const { loading, error } = useNodeList();
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

  const quicFilter =
    vpnMode === 'wg' && node === 'entry' && backendFlags.quic && quic;

  const navigate = useNavigate();
  const { t } = useTranslation('nodeLocation');

  const { filter, nodes, gateways } = useFilterList(node);
  const deferredNodes = useDeferredValue(nodes);
  const deferredGateways = useDeferredValue(gateways);
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (searchRef.current) searchRef.current.focus();
  }, []);

  const handleSelect = async (selected: SelectedUiNode) => {
    const selectedNode = uiNodeToSelectedNode(selected);
    if (
      selectedNode.type === 'gateway' &&
      (selected.isSelected === 'exit' || selected.isSelected === 'entry')
    ) {
      return;
    }

    await kvSet(
      node === 'entry' ? 'entry-node' : 'exit-node',
      uiNodeToSelectedNode(selected),
    );
    dispatch({
      type: 'set-node',
      payload: { hop: node, node: selectedNode },
    });
    navigate(routes.root);
    resetSaved(node);
  };

  const handleNodeDetails = (gateway: UiGateway) => {
    navigate(routes.nodeDetails, {
      state: { gateway, hop: node, resetScroll: true },
    });
    setFocused(node, { type: 'gateway', key: gateway.id });
    // if the picked gateway's country node is not expanded, ie; while filtering
    // expand it, so it can be restored and scrolled to when navigating back
    // to the node list
    addToExpanded(node, gateway.country.code);
  };

  const onSearchChange = (value: string) => {
    setSearch(node, value);
    filter(value);
  };

  if (error) {
    return (
      <PageAnim
        className="h-full flex flex-col"
        data-testid="node-error-container"
      >
        <div
          className="w-4/5 h-2/3 overflow-auto break-words text-center"
          data-testid="node-error-message"
        >
          <p
            className="text-aphrodisiac font-medium"
            data-testid="node-error-title"
          >
            An error occurred
          </p>
          <p
            className="text-base font-mono"
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
        className="h-full flex flex-col"
        data-testid={`node-container-${node}`}
      >
        <div
          className="w-full px-6 mt-6 mb-6"
          data-testid="node-search-container"
        >
          {quicFilter && (
            <p className="text-sm text-iron dark:text-bombay mb-6 select-none">
              <Trans
                i18nKey="quic-filter-note"
                ns="nodeLocation"
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
            label={t('input-label')}
            clearable
            value={search || ''}
          />
        </div>
        {loading && (
          <motion.div
            className="flex justify-center text-base text-iron dark:text-bombay mt-4"
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
            expanded={expanded}
            focused={focused}
          />
        )}
      </PageAnim>
    </>
  );
}

export default Node;
