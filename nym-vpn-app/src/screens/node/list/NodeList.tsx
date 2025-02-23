import { useState } from 'react';
import * as Accordion from '@radix-ui/react-accordion';
import clsx from 'clsx';
import { UiCountry, UiGateway, UiGatewaysByCountry } from '../../../contexts';
import { VpnMode } from '../../../types';

export type NodeListProps = {
  nodes: UiGatewaysByCountry[];
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

function NodeList({ nodes, onSelect, vpnMode }: NodeListProps) {
  return (
    <Accordion.Root className="w-full overflow-x-hidden" type="single" collapsible>
      {nodes.map((country) => {
        const {
          i18n,
          isSelected,
          gateways,
          country: { code },
        } = country;

        return (
          <Accordion.Item
            key={code}
            className={clsx('')}
            value={country.country.code}
          >
            <AccordionTrigger>
              <div className="text-2xl font-bold text-malachite">{`[${code}] ${i18n} selected: ${isSelected}`}</div>
            </AccordionTrigger>
            <Accordion.Content>
              {gateways.map((gateway) => (
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
                    {vpnMode === 'Mixnet'
                      ? ` ${gateway.mxScore}`
                      : ` ${gateway.wgScore}`}
                  </span>
                </div>
              ))}
            </Accordion.Content>
          </Accordion.Item>
        );
      })}
    </Accordion.Root>
  );
}

export default NodeList;
