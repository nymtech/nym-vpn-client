import { useShallow } from 'zustand/react/shallow';
import { useAppStore } from '../../store/index';
import { Score } from '../../types/index';
import { ScoreIndicator } from '../node/ScoreIndicator';

export function ScoreIndicatorContainer({ score }: { score?: Score }) {
  const { state, gatewaySelectionAlgorithmConfig } = useAppStore(
    useShallow((s) => ({
      state: s.state,
      gatewaySelectionAlgorithmConfig: s.gatewaySelectionAlgorithmConfig,
    })),
  );

  if (
    gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm === 'auto' &&
    state !== 'connected' &&
    state !== 'connecting'
  ) {
    return <ScoreIndicator score={'high'} />;
  }

  return <ScoreIndicator score={score} />;
}
