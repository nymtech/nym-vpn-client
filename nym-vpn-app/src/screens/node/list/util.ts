import { Gateway, VpnMode } from '../../../types';

export function getScoreIcon(gw: Gateway, vpnMode: VpnMode) {
  const score = vpnMode === 'mixnet' ? gw.mxScore : gw.wgScore;
  switch (score) {
    case 'none':
      return ['signal_cellular_alt_1_bar', 'text-iron'];
    case 'low':
      return ['signal_cellular_alt_1_bar', 'text-aphrodisiac'];
    case 'medium':
      return ['signal_cellular_alt_2_bar', 'text-king-nacho'];
    case 'high':
      return ['signal_cellular_alt', 'text-malachite'];
  }
}
