import * as Accordion from '@radix-ui/react-accordion';
import clsx from 'clsx';
import { UiCountry, UiGateway, UiGatewaysByCountry } from '../../../contexts';
import { VpnMode } from '../../../types';

export type NodeListProps = {
  nodes: UiGatewaysByCountry[];
  gateways: UiGateway[];
  onSelect: (node: UiCountry | UiGateway) => void;
  vpnMode: VpnMode;
};

type AccordionTriggerProps = {
  children: React.ReactNode;
  ref?: React.Ref<never>;
};
const AccordionTrigger = ({ children, ref }: AccordionTriggerProps) => (
  <Accordion.Header className="flex">
    <Accordion.Trigger className={clsx('')} ref={ref}>
      {children}
    </Accordion.Trigger>
  </Accordion.Header>
);

type GatewayItemProps = {
  gateway: UiGateway;
  onSelect: (node: UiGateway) => void;
  vpnMode: VpnMode;
};

function NodeList({ nodes, gateways, onSelect, vpnMode }: NodeListProps) {
  const GatewayItem = ({ gateway }: GatewayItemProps) => (
    <div
      className="ml-4 text-liquid-lava"
      key={gateway.id}
      onClick={() => onSelect(gateway)}
    >
      <span className="text-cornflower font-mono">
        {gateway.id.slice(0, 6)}
      </span>
      <span>{` ${gateway.name.slice(0, 30)}`}</span>
      <span className="text-cornflower font-mono">
        {vpnMode === 'Mixnet' ? ` ${gateway.mxScore}` : ` ${gateway.wgScore}`}
      </span>
    </div>
  );

  return (
    <>
      <Accordion.Root
        className="w-full overflow-x-hidden"
        type="single"
        collapsible
      >
        {nodes.map(({ i18n, isSelected, gateways, country: { code } }) => (
          <Accordion.Item key={code} className={clsx('')} value={code}>
            <AccordionTrigger>
              <div className="text-2xl font-bold text-malachite">{`[${code}] ${i18n} selected: ${isSelected}`}</div>
            </AccordionTrigger>
            <Accordion.Content>
              {gateways.map((gateway) => (
                <GatewayItem
                  key={gateway.id}
                  gateway={gateway}
                  onSelect={onSelect}
                  vpnMode={vpnMode}
                />
              ))}
            </Accordion.Content>
          </Accordion.Item>
        ))}
      </Accordion.Root>
      <div className={clsx('mt-6')}>
        {gateways.length > 0 &&
          gateways.map((gateway) => (
            <GatewayItem
              key={gateway.id}
              gateway={gateway}
              onSelect={onSelect}
              vpnMode={vpnMode}
            />
          ))}
      </div>
    </>
  );
}

export default NodeList;
