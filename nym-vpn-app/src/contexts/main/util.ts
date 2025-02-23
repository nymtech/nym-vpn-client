import { GatewayType } from '../../types';
import { CKey } from '../../cache';

export function gwTypeToCacheKey(type: GatewayType): CKey {
  if (type === 'wg') return 'cache-wg-gateways';
  return `cache-${type}-gateways`;
}
