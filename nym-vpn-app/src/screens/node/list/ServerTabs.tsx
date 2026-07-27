import { useTranslation } from 'react-i18next';
import { NodeHop } from '../../../types';
import { MsIcon, SegmentedToggle, SegmentedToggleItem } from '../../../ui';

export type ServerTab = 'all' | 'favorites';

// Segmented control that filters the node list between all servers and
// favorites (the design's third "Recent" tab is deferred). Shares the visual
// with the home "Fast | Mixnet" toggle via SegmentedToggle.
function ServerTabs({
  value,
  onChange,
  hop,
}: {
  value: ServerTab;
  onChange: (value: ServerTab) => void;
  hop: NodeHop;
}) {
  const { t } = useTranslation('node-location');

  const items: SegmentedToggleItem<ServerTab>[] = [
    {
      id: 'all',
      label: t('favorites.tab-all'),
      icon: <MsIcon icon="list" className="text-base!" />,
      'data-testid': 'server-tab-all',
    },
    {
      id: 'favorites',
      label: t('favorites.tab-favorites'),
      icon: (
        <MsIcon
          icon="star"
          filled={value === 'favorites'}
          className="text-base!"
        />
      ),
      'data-testid': 'server-tab-favorites',
    },
  ];

  return (
    <SegmentedToggle
      items={items}
      value={value}
      onChange={onChange}
      layoutId={`server-tabs-pill-${hop}`}
      data-testid="server-tabs"
    />
  );
}

export default ServerTabs;
