import * as Accordion from '@radix-ui/react-accordion';
import { motion } from 'motion/react';
import clsx from 'clsx';
import {
  SelectedKind,
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
} from '../../../contexts';
import { NodeHop, VpnMode } from '../../../types';
import CountryInfo from './CountryInfo';
import GatewayItem from './GatewayItem';
import FoldButton from './FoldButton';

export type NodeListProps = {
  nodes: UiGatewaysByCountry[];
  gateways: UiGateway[];
  onSelect: (node: UiCountry | UiGateway) => void;
  onNodeDetails: (node: UiGateway | UiCountry) => void;
  node: NodeHop;
  vpnMode: VpnMode;
};

function NodeList({
  nodes,
  gateways,
  onSelect,
  node,
  vpnMode,
  onNodeDetails,
}: NodeListProps) {
  const handleCountrySelect = (
    country: UiCountry,
    isSelected: SelectedKind,
    gwCount: number,
  ) => {
    if (isSelected && isSelected !== node && gwCount <= 1) {
      // don't allow selecting a country if it has only one gateway,
      // and it's already selected by the other hop
      return;
    }
    if (isSelected !== node && isSelected !== 'entry-and-exit') {
      onSelect(country);
    }
  };

  return (
    <>
      <Accordion.Root
        className="w-full flex flex-col gap-3"
        type="multiple"
        data-testid="node-list-accordion"
      >
        {nodes.map(({ i18n, isSelected, gateways, country }) => (
          <Accordion.Item
            key={country.code}
            value={country.code}
            data-testid={`country-accordion-item-${country.code}`}
          >
            <div
              className={clsx(
                'flex flex-row justify-between',
                ' bg-white dark:bg-charcoal',
                'hover:bg-white/60 dark:hover:bg-charcoal/85',
              )}
              data-testid={`country-header-${country.code}`}
            >
              <div
                className={clsx(
                  'w-1.5 rounded-r-sm',
                  (isSelected === node || isSelected === 'entry-and-exit') &&
                    'bg-malachite',
                  isSelected && isSelected !== node && 'bg-iron',
                )}
                data-testid={`country-selection-indicator-${country.code}`}
                data-selected={isSelected ? isSelected : 'none'}
              />
              <div
                className={clsx('grow overflow-hidden truncate py-2')}
                onClick={() =>
                  handleCountrySelect(country, isSelected, gateways.length)
                }
                data-testid={`country-select-area-${country.code}`}
              >
                <CountryInfo
                  country={country}
                  name={i18n}
                  gwCount={gateways.length}
                />
              </div>
              <Accordion.Header
                className="flex py-2"
                data-testid={`country-accordion-header-${country.code}`}
              >
                <Accordion.Trigger asChild>
                  <FoldButton />
                </Accordion.Trigger>
              </Accordion.Header>
            </div>
            <Accordion.Content
              data-testid={`country-accordion-content-${country.code}`}
            >
              <motion.div
                initial={{ opacity: 0, translateY: -4 }}
                animate={{ opacity: 1, translateY: 0 }}
                transition={{ duration: 0.1, ease: 'easeIn' }}
                className="flex flex-col gap-2"
                data-testid={`country-gateways-container-${country.code}`}
              >
                {gateways.map((gateway) => (
                  <GatewayItem
                    key={gateway.id}
                    node={node}
                    gateway={gateway}
                    onSelect={onSelect}
                    onNodeDetails={onNodeDetails}
                    vpnMode={vpnMode}
                  />
                ))}
              </motion.div>
            </Accordion.Content>
          </Accordion.Item>
        ))}
      </Accordion.Root>
      <div className={clsx('mt-6')} data-testid="standalone-gateways-container">
        {gateways.length > 0 &&
          gateways.map((gateway) => (
            <motion.div
              key={gateway.id}
              initial={{ opacity: 0, translateX: -4 }}
              animate={{ opacity: 1, translateX: 0 }}
              transition={{ duration: 0.1, ease: 'easeOut' }}
              className="flex flex-col gap-2"
              data-testid={`standalone-gateway-${gateway.id.substring(0, 8)}`}
            >
              <GatewayItem
                node={node}
                gateway={gateway}
                onSelect={onSelect}
                vpnMode={vpnMode}
                onNodeDetails={onNodeDetails}
              />
            </motion.div>
          ))}
      </div>
    </>
  );
}

export default NodeList;
