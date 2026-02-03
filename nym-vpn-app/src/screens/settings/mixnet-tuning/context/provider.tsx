import { useMemo, useReducer } from 'react';
import { MixnetTrafficConfig } from '../../../../types';
import { MixnetTrafficConfigContext } from './context';
import { DEFAULT_MIXNET_TRAFFIC_CONFIG, reducer } from './reducer';

function MixnetTrafficConfigProvider({
  children,
  initialConfig,
}: {
  children: React.ReactNode;
  initialConfig: MixnetTrafficConfig;
}) {
  const [state, dispatch] = useReducer(reducer, initialConfig, (config) => ({
    poissonParameterForLoopCoverStream:
      config.poissonParameterForLoopCoverStream ??
      DEFAULT_MIXNET_TRAFFIC_CONFIG.poissonParameterForLoopCoverStream!,
    averagePacketDelay:
      config.averagePacketDelay ??
      DEFAULT_MIXNET_TRAFFIC_CONFIG.averagePacketDelay!,
    messageSendingAverageDelay:
      config.messageSendingAverageDelay ??
      DEFAULT_MIXNET_TRAFFIC_CONFIG.messageSendingAverageDelay!,
    disablePoissonRate: config.disablePoissonRate,
    disableBackgroundCoverTraffic: config.disableBackgroundCoverTraffic,
  }));

  const hasUnsavedSettings = useMemo(() => {
    return (
      state.poissonParameterForLoopCoverStream !==
        initialConfig.poissonParameterForLoopCoverStream ||
      state.averagePacketDelay !== initialConfig.averagePacketDelay ||
      state.messageSendingAverageDelay !==
        initialConfig.messageSendingAverageDelay ||
      state.disablePoissonRate !== initialConfig.disablePoissonRate ||
      state.disableBackgroundCoverTraffic !==
        initialConfig.disableBackgroundCoverTraffic
    );
  }, [state, initialConfig]);

  const hasSettingsOtherThanDefaults = useMemo(() => {
    return (
      state.poissonParameterForLoopCoverStream !==
        DEFAULT_MIXNET_TRAFFIC_CONFIG.poissonParameterForLoopCoverStream ||
      state.averagePacketDelay !==
        DEFAULT_MIXNET_TRAFFIC_CONFIG.averagePacketDelay ||
      state.messageSendingAverageDelay !==
        DEFAULT_MIXNET_TRAFFIC_CONFIG.messageSendingAverageDelay ||
      state.disablePoissonRate !==
        DEFAULT_MIXNET_TRAFFIC_CONFIG.disablePoissonRate ||
      state.disableBackgroundCoverTraffic !==
        DEFAULT_MIXNET_TRAFFIC_CONFIG.disableBackgroundCoverTraffic
    );
  }, [state]);

  return (
    <MixnetTrafficConfigContext.Provider
      value={{
        state,
        dispatch,
        hasUnsavedSettings,
        hasSettingsOtherThanDefaults,
      }}
    >
      {children}
    </MixnetTrafficConfigContext.Provider>
  );
}

export default MixnetTrafficConfigProvider;
