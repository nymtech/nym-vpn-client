import { CKey } from '../../cache';
import { GatewayType, GatewaysByCountry, SelectedNode } from '../../types';
import { GatewaysState } from './types';

export function gwTypeToCacheKey(type: GatewayType): CKey {
  if (type === 'wg') return 'cache-wg-gateways';
  return `cache-${type}-gateways`;
}

type GatewaysKey = keyof Pick<GatewaysState, 'mxEntry' | 'mxExit' | 'wg'>;
type LoadingKey = keyof Pick<
  GatewaysState,
  'mxEntryLoading' | 'mxExitLoading' | 'wgLoading'
>;
type ErrorKey = keyof Pick<
  GatewaysState,
  'mxEntryError' | 'mxExitError' | 'wgError'
>;

export function getStateProps(type: GatewayType): {
  gateways: GatewaysKey;
  loading: LoadingKey;
  error: ErrorKey;
} {
  if (type === 'mx-entry') {
    return {
      gateways: 'mxEntry',
      loading: 'mxEntryLoading',
      error: 'mxEntryError',
    };
  }
  if (type === 'mx-exit') {
    return {
      gateways: 'mxExit',
      loading: 'mxExitLoading',
      error: 'mxExitError',
    };
  }
  return {
    gateways: 'wg',
    loading: 'wgLoading',
    error: 'wgError',
  };
}

// Check if a node exists in the gateways list
export function exists(
  { type, node }: SelectedNode,
  gateways: GatewaysByCountry[],
) {
  switch (type) {
    case 'country':
      return gateways.some((g) => g.country.code === node.code);
    case 'region':
      return gateways.some((g) => g.regions.some((r) => r.name === node.name));
    case 'gateway':
      return gateways.some((g) => g.gateways.some((gw) => gw.id === node.id));
    default:
      return false;
  }
}
