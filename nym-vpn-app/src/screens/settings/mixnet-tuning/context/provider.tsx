import { useCallback, useMemo, useReducer } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { MixnetTrafficConfig } from '../../../../types';
import { useAppStore } from '../../../../store';
import { MixnetTrafficConfigContext } from './context';
import { reducer } from './reducer';

function MixnetTrafficConfigProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const { mixnetTrafficConfig, mixnetTrafficDefaults } = useAppStore(
    useShallow((s) => ({
      mixnetTrafficConfig: s.mixnetTrafficConfig,
      mixnetTrafficDefaults: s.mixnetTrafficDefaults,
    })),
  );

  const [state, dispatch] = useReducer(
    reducer,
    mixnetTrafficConfig,
    (config) => ({
      poissonParameterForLoopCoverStream:
        config.poissonParameterForLoopCoverStream === null
          ? mixnetTrafficDefaults.defaultBackgroundTraffic.value
          : config.poissonParameterForLoopCoverStream,
      averagePacketDelay:
        config.averagePacketDelay === null
          ? mixnetTrafficDefaults.mixingDelay.defaultValue
          : config.averagePacketDelay,
      messageSendingAverageDelay:
        config.messageSendingAverageDelay === null
          ? mixnetTrafficDefaults.defaultContinuousTraffic.value
          : config.messageSendingAverageDelay,
      disablePoissonRate: config.disablePoissonRate,
      disableBackgroundCoverTraffic: config.disableBackgroundCoverTraffic,
      minMixnodePerformance: 0,
      minGatewayMixnetPerformance: 0,
    }),
  );

  const defaultState = useMemo(() => {
    return {
      poissonParameterForLoopCoverStream:
        mixnetTrafficDefaults.mixingDelay.defaultValue,
      averagePacketDelay: mixnetTrafficDefaults.mixingDelay.defaultValue,
      messageSendingAverageDelay:
        mixnetTrafficDefaults.mixingDelay.defaultValue,
      disablePoissonRate: mixnetTrafficDefaults.disablePoissonRate,
      disableBackgroundCoverTraffic: mixnetTrafficDefaults.disablePoissonRate,
      minGatewayMixnetPerformance: 0,
      minMixnodePerformance: 0,
    };
  }, [mixnetTrafficDefaults]);

  const hasUnsavedSettings = useMemo(() => {
    return (
      state.poissonParameterForLoopCoverStream !==
        mixnetTrafficConfig.poissonParameterForLoopCoverStream ||
      state.averagePacketDelay !== mixnetTrafficConfig.averagePacketDelay ||
      state.messageSendingAverageDelay !==
        mixnetTrafficConfig.messageSendingAverageDelay ||
      state.disablePoissonRate !== mixnetTrafficConfig.disablePoissonRate ||
      state.disableBackgroundCoverTraffic !==
        mixnetTrafficConfig.disableBackgroundCoverTraffic
    );
  }, [state, mixnetTrafficConfig]);

  const hasSettingsOtherThanDefaults = useMemo(() => {
    return (
      state.poissonParameterForLoopCoverStream !==
        mixnetTrafficDefaults.mixingDelay.defaultValue ||
      state.averagePacketDelay !==
        mixnetTrafficDefaults.mixingDelay.defaultValue ||
      state.messageSendingAverageDelay !==
        mixnetTrafficDefaults.mixingDelay.defaultValue ||
      state.disablePoissonRate !== mixnetTrafficDefaults.disablePoissonRate ||
      state.disableBackgroundCoverTraffic !==
        mixnetTrafficDefaults.allBackgroundTraffic.length > 0
    );
  }, [state, mixnetTrafficDefaults]);

  const updateField = useCallback(
    (field: keyof MixnetTrafficConfig, value: number | boolean) => {
      dispatch({ type: 'update-field', field, value });
    },
    [dispatch],
  );

  const restoreDefaults = useCallback(() => {
    dispatch({ type: 'update-fields', state: defaultState });
  }, [dispatch, defaultState]);

  const continuousItems = useMemo(
    () =>
      mixnetTrafficDefaults.allContinuousTraffic.map((item) => ({
        value: item.value,
        label: item.throughput,
      })),
    [mixnetTrafficDefaults],
  );

  const backgroundCoverItems = useMemo(
    () =>
      mixnetTrafficDefaults.allBackgroundTraffic.map((item) => ({
        value: item.value,
        label: item.multiplier,
      })),
    [mixnetTrafficDefaults],
  );

  const mixingDelay = useMemo(
    () => ({
      minValue: mixnetTrafficDefaults.mixingDelay.minValue,
      maxValue: mixnetTrafficDefaults.mixingDelay.maxValue,
      defaultValue: mixnetTrafficDefaults.mixingDelay.defaultValue,
    }),
    [mixnetTrafficDefaults],
  );

  return (
    <MixnetTrafficConfigContext.Provider
      value={{
        state,
        hasUnsavedSettings,
        hasSettingsOtherThanDefaults,
        updateField,
        restoreDefaults,
        continuousItems,
        backgroundCoverItems,
        mixingDelay,
      }}
    >
      {children}
    </MixnetTrafficConfigContext.Provider>
  );
}

export default MixnetTrafficConfigProvider;
