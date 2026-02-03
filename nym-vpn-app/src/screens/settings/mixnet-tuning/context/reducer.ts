import { MixnetTrafficConfig } from '../../../../types';

export type MixnetTrafficConfigAction =
  | {
      type: 'update-field';
      field: keyof MixnetTrafficConfig;
      value: number | boolean;
    }
  | { type: 'restore-defaults' };

export const DEFAULT_MIXNET_TRAFFIC_CONFIG: NonNullable<MixnetTrafficConfig> = {
  poissonParameterForLoopCoverStream: 200,
  averagePacketDelay: 15,
  messageSendingAverageDelay: 20,
  disablePoissonRate: false,
  disableBackgroundCoverTraffic: false,
  minMixnodePerformance: null,
  minGatewayMixnetPerformance: null,
} as const;

export function reducer(
  state: MixnetTrafficConfig,
  action: MixnetTrafficConfigAction,
): NonNullable<MixnetTrafficConfig> {
  switch (action.type) {
    case 'update-field':
      return {
        ...state,
        [action.field]: action.value,
      };
    case 'restore-defaults':
      return DEFAULT_MIXNET_TRAFFIC_CONFIG;
    default:
      return state;
  }
}
