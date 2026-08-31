import { useState } from 'react';
import { Tabs } from '@base-ui/react';
import { useLocation, useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import { useShallow } from 'zustand/react/shallow';
import { NodeHop, isCountry, isGateway, isRegion } from '../../types';
import { useNodeListState } from '../../store/nodeListState';
import { useAppStore, useLookupGw } from '../../store';
import { regionToCountryCode } from '../home/util';
import Node from './Node';

type LocationState = {
  tab?: NodeHop;
};

function NodeLocation() {
  const location = useLocation();
  const navigate = useNavigate();
  const { entryNode, exitNode } = useAppStore(
    useShallow((s) => ({
      entryNode: s.entryNode,
      exitNode: s.exitNode,
    })),
  );
  const locationState = location.state as LocationState | null;
  const initialTab: NodeHop = locationState?.tab ?? 'exit';
  const [activeTab, setActiveTab] = useState<NodeHop>(initialTab);
  const { reset, setFocused, addToExpanded } = useNodeListState();
  const lookupGw = useLookupGw();
  const { t } = useTranslation('node-location');

  const focusSelectedNode = (hop: NodeHop) => {
    const userSelectedNode = hop === 'entry' ? entryNode : exitNode;

    if (isCountry(userSelectedNode)) {
      setFocused(hop, {
        type: 'country',
        key: userSelectedNode.country.code,
      });
    } else if (isRegion(userSelectedNode)) {
      const code = regionToCountryCode(userSelectedNode.region);
      if (code) {
        addToExpanded(hop, code.toUpperCase());
        setFocused(hop, { type: 'region', key: userSelectedNode.region });
      }
    } else if (isGateway(userSelectedNode)) {
      setFocused(hop, { type: 'gateway', key: userSelectedNode.gateway.id });
      const gw = lookupGw(userSelectedNode.gateway.id, hop);
      if (gw) {
        addToExpanded(hop, gw.country.code.toUpperCase());
        if (gw.country.code.toLowerCase() === 'us') {
          addToExpanded(hop, gw.location.region);
        }
      }
      addToExpanded(hop, userSelectedNode.gateway.id);
    }
  };

  const handleTabChange = (value: unknown) => {
    if (value !== 'entry' && value !== 'exit') return;
    const newTab = value;
    if (newTab === activeTab) return;
    reset(newTab);
    focusSelectedNode(newTab);
    setActiveTab(newTab);
    navigate('.', { replace: true, state: { tab: newTab } });
  };

  return (
    <Tabs.Root
      value={activeTab}
      onValueChange={handleTabChange}
      className="flex h-full flex-col"
    >
      <Tabs.List className="bg-surface-sunken dark:bg-surface-bg flex px-4 select-none">
        <Tabs.Tab
          value="entry"
          className="group text-text-secondary data-active:text-text-primary flex flex-1 flex-col items-center gap-2 py-2 text-base font-medium tracking-tight focus-visible:outline-none"
        >
          <span>{t('tab-entry')}</span>
          <span className="bg-surface-hair group-data-active:bg-brand-primary h-[1.5px] w-full" />
        </Tabs.Tab>
        <Tabs.Tab
          value="exit"
          className="group text-text-secondary data-active:text-text-primary flex flex-1 flex-col items-center gap-2 py-2 text-base font-medium tracking-tight focus-visible:outline-none"
        >
          <span>{t('tab-exit')}</span>
          <span className="bg-surface-hair group-data-active:bg-brand-primary h-[1.5px] w-full" />
        </Tabs.Tab>
      </Tabs.List>
      <Tabs.Panel value="entry" className="min-h-0 flex-1">
        <Node node="entry" />
      </Tabs.Panel>
      <Tabs.Panel value="exit" className="min-h-0 flex-1">
        <Node node="exit" />
      </Tabs.Panel>
    </Tabs.Root>
  );
}

export default NodeLocation;
