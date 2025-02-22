import { GatewayType } from '../../types';
import { CKey } from '../../cache';

export function gwTypeToCacheKey(type: GatewayType): CKey {
  if (type === 'wg') return 'cache-wg-gateways';
  return `cache-${type}-gateways`;
}

export function gwTypeToDispatchError(type: GatewayType) {
  if (type === 'mx-entry') return 'set-mx-entry-gateways-error';
  if (type === 'mx-exit') return 'set-mx-exit-gateways-error';
  return 'set-wg-gateways-error';
}

export function gwTypeToDispatchSet(type: GatewayType) {
  if (type === 'mx-entry') return 'set-mx-entry-gateways';
  if (type === 'mx-exit') return 'set-mx-exit-gateways';
  return 'set-wg-gateways';
}
