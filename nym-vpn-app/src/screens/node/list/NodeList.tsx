import { ReactNode, memo, useCallback, useEffect, useRef } from 'react';
import { dequal } from 'dequal';
import { Accordion } from '@base-ui-components/react';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import {
  Focused,
  SelectedKind,
  SelectedUiNode,
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
  UiRegion,
  useMainState,
  useNodeListState,
} from '../../../contexts';
import { NodeHop, VpnMode } from '../../../types';
import GatewayItem from './GatewayItem';
import RowHeader from './RowHeader';

export type NodeListProps = {
  nodes: UiGatewaysByCountry[];
  gateways: UiGateway[];
  onSelect: (node: SelectedUiNode) => void;
  onNodeDetails: (node: UiGateway) => void;
  hop: NodeHop;
  vpnMode: VpnMode;
  expanded: string[];
  focused: Focused | null;
};

const NodeList = memo(function NodeList({
  nodes,
  gateways,
  onSelect,
  hop,
  vpnMode,
  onNodeDetails,
  expanded,
  focused,
}: NodeListProps) {
  const { backendFlags, quic } = useMainState();
  const { setExpanded } = useNodeListState();
  const { t } = useTranslation('nodeLocation');

  const countriesRef = useRef<Map<string, HTMLDivElement>>(null);
  const regionsRef = useRef<Map<string, HTMLDivElement>>(null);
  const gatewaysRef = useRef<Map<string, HTMLDivElement>>(null);
  const quicFilter =
    vpnMode === 'wg' && hop === 'entry' && backendFlags.quic && quic;

  const getMap = (type: 'country' | 'region' | 'gateway') => {
    if (type === 'country') {
      if (!countriesRef.current) {
        countriesRef.current = new Map();
      }
      return countriesRef.current;
    }
    if (type === 'region') {
      if (!regionsRef.current) {
        regionsRef.current = new Map();
      }
      return regionsRef.current;
    }
    if (type === 'gateway') {
      if (!gatewaysRef.current) {
        gatewaysRef.current = new Map();
      }
      return gatewaysRef.current;
    }
  };

  const setRef = (
    type: 'country' | 'region' | 'gateway',
    key: string,
    node: HTMLDivElement | null,
  ) => {
    if (!node) {
      return;
    }
    const map = getMap(type);
    map?.set(key, node);

    return () => {
      map?.delete(key);
    };
  };

  const scrollToNode = useCallback(
    (type: 'country' | 'region' | 'gateway', key: string) => {
      const map = getMap(type);
      const node = map?.get(key);
      node?.scrollIntoView({
        behavior: 'smooth',
        block: 'center',
        inline: 'center',
      });
    },
    [],
  );

  const handleLocationSelect = (
    location: UiCountry | UiRegion,
    isSelected: SelectedKind,
    gwCount: number,
  ) => {
    if (isSelected && isSelected !== hop && gwCount <= 1) {
      // don't allow selecting a country if it has only one gateway,
      // and it's already selected by the other hop
      return;
    }
    if (isSelected !== hop && isSelected !== 'entry-and-exit') {
      onSelect(location);
    }
  };

  const onValueChange = (value: string[]) => {
    setExpanded(hop, value);
  };

  useEffect(() => {
    let timeoutId: NodeJS.Timeout;
    if (focused) {
      timeoutId = setTimeout(() => {
        scrollToNode(focused.type, focused.key);
      }, 20);
    }

    return () => clearTimeout(timeoutId);
  }, [focused, scrollToNode]);

  const PanelContent = ({
    children,
    animate = false,
  }: {
    children: ReactNode;
    animate?: boolean;
  }) => (
    <motion.div
      initial={animate && { opacity: 0, translateY: -4 }}
      animate={animate && { opacity: 1, translateY: 0 }}
      transition={animate ? { duration: 0.1, ease: 'easeIn' } : undefined}
      className="flex flex-col gap-2"
    >
      {children}
    </motion.div>
  );

  return (
    <>
      <Accordion.Root
        className="w-full flex flex-col gap-3"
        data-testid="node-list-accordion"
        value={expanded}
        onValueChange={onValueChange}
        openMultiple
      >
        {nodes.map(({ i18n, isSelected, gateways, country, regions }) => (
          <Accordion.Item
            key={country.code}
            value={country.code}
            ref={(node) => setRef('country', country.code, node)}
            data-testid={`country-accordion-item-${country.code}`}
          >
            <RowHeader
              hop={hop}
              isSelected={isSelected}
              node={country}
              i18n={i18n}
              onClick={() =>
                handleLocationSelect(country, isSelected, gateways.length)
              }
              gwCount={gateways.length}
            />
            <Accordion.Panel
              data-testid={`country-accordion-content-${country.code}`}
              className="w-full flex flex-col gap-3"
            >
              {country.code.toLowerCase() === 'us' ? (
                regions.map((region) => (
                  <Accordion.Item
                    className="first:pt-3"
                    key={region.name}
                    value={region.name}
                    ref={(node) => setRef('region', region.name, node)}
                  >
                    <RowHeader
                      hop={hop}
                      isSelected={region.isSelected}
                      node={region}
                      i18n={i18n}
                      onClick={() => {
                        handleLocationSelect(
                          region,
                          region.isSelected,
                          region.gateways.length,
                        );
                      }}
                      gwCount={region.gateways.length}
                      sub
                    />
                    <Accordion.Panel>
                      <PanelContent>
                        {region.gateways.map((gateway) => (
                          <GatewayItem
                            key={gateway.id}
                            ref={(node) => setRef('gateway', gateway.id, node)}
                            node={hop}
                            gateway={gateway}
                            onSelect={onSelect}
                            onNodeDetails={onNodeDetails}
                            vpnMode={vpnMode}
                            quicLabel={quicFilter}
                          />
                        ))}
                      </PanelContent>
                    </Accordion.Panel>
                  </Accordion.Item>
                ))
              ) : (
                <PanelContent>
                  {gateways.map((gateway) => (
                    <GatewayItem
                      key={gateway.id}
                      ref={(node) => setRef('gateway', gateway.id, node)}
                      node={hop}
                      gateway={gateway}
                      onSelect={onSelect}
                      onNodeDetails={onNodeDetails}
                      vpnMode={vpnMode}
                      quicLabel={quicFilter}
                    />
                  ))}
                </PanelContent>
              )}
            </Accordion.Panel>
          </Accordion.Item>
        ))}
      </Accordion.Root>
      {gateways.length > 0 && (
        <div className="mt-2" data-testid="standalone-gateways-container">
          <h3 className="text-iron dark:text-bombay px-4 py-6 truncate">
            {t('search-other-nodes')}
          </h3>
          {gateways.map((gateway) => (
            <motion.div
              key={gateway.id}
              initial={{ opacity: 0, translateX: -4 }}
              animate={{ opacity: 1, translateX: 0 }}
              transition={{ duration: 0.1, ease: 'easeOut' }}
              className="flex flex-col gap-2"
              data-testid={`standalone-gateway-${gateway.id.substring(0, 8)}`}
            >
              <GatewayItem
                node={hop}
                gateway={gateway}
                onSelect={onSelect}
                vpnMode={vpnMode}
                onNodeDetails={onNodeDetails}
                quicLabel={quicFilter}
                inSearchResult
              />
            </motion.div>
          ))}
        </div>
      )}
    </>
  );
}, arePropsEqual);

export default NodeList;

function arePropsEqual(
  oldProps: NodeListProps,
  newProps: NodeListProps,
): boolean {
  if (oldProps.hop !== newProps.hop) return false;
  if (oldProps.vpnMode !== newProps.vpnMode) return false;
  if (oldProps.gateways.length !== newProps.gateways.length) return false;
  if (oldProps.nodes.length !== newProps.nodes.length) return false;
  if (!dequal(oldProps.expanded, newProps.expanded)) return false;
  if (!dequal(oldProps.focused, newProps.focused)) return false;
  if (!dequal(oldProps.gateways, newProps.gateways)) return false;
  if (!dequal(oldProps.nodes, newProps.nodes)) return false;
  return true;
}
