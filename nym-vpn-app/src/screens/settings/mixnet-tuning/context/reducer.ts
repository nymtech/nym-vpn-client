import { MixnetTrafficConfig } from '../../../../types';

export type MixnetTrafficConfigState = {
  poissonParameterForLoopCoverStream: number;
  averagePacketDelay: number;
  messageSendingAverageDelay: number;
  disablePoissonRate: boolean;
  disableBackgroundCoverTraffic: boolean;
  minMixnodePerformance: number | null;
  minGatewayMixnetPerformance: number | null;
};

export type MixnetTrafficConfigAction =
  | {
      type: 'update-field';
      field: keyof MixnetTrafficConfigState;
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
  state: MixnetTrafficConfigState,
  action: MixnetTrafficConfigAction,
): MixnetTrafficConfigState {
  switch (action.type) {
    case 'update-field':
      return {
        ...state,
        [action.field]: action.value,
      };
    case 'restore-defaults':
      return {
        poissonParameterForLoopCoverStream:
          DEFAULT_MIXNET_TRAFFIC_CONFIG.poissonParameterForLoopCoverStream!,
        averagePacketDelay: DEFAULT_MIXNET_TRAFFIC_CONFIG.averagePacketDelay!,
        messageSendingAverageDelay:
          DEFAULT_MIXNET_TRAFFIC_CONFIG.messageSendingAverageDelay!,
        disablePoissonRate: DEFAULT_MIXNET_TRAFFIC_CONFIG.disablePoissonRate,
        disableBackgroundCoverTraffic:
          DEFAULT_MIXNET_TRAFFIC_CONFIG.disableBackgroundCoverTraffic,
        minMixnodePerformance:
          DEFAULT_MIXNET_TRAFFIC_CONFIG.minMixnodePerformance,
        minGatewayMixnetPerformance:
          DEFAULT_MIXNET_TRAFFIC_CONFIG.minGatewayMixnetPerformance,
      };
    default:
      return state;
  }
}
