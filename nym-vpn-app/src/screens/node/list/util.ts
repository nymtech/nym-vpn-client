import { Gateway, Score, VpnMode } from '../../../types';

export function getScoreIcon(gw: Gateway, vpnMode: VpnMode) {
  const score = vpnMode === 'mixnet' ? gw.mxScore : gw.wgScore;
  switch (score) {
    case 'offline':
      return ['signal_cellular_alt_1_bar', 'text-iron'];
    case 'low':
      return ['signal_cellular_alt_1_bar', 'text-aphrodisiac'];
    case 'medium':
      return ['signal_cellular_alt_2_bar', 'text-king-nacho'];
    case 'high':
      return ['signal_cellular_alt', 'text-malachite'];
  }
}

const scoreOrder: Record<Score, number> = {
  offline: 0,
  low: 1,
  medium: 2,
  high: 3,
};

export function sortByScore(a: Score, b: Score): number {
  if (a === b) {
    return 0;
  }
  return scoreOrder[b] - scoreOrder[a];
}
