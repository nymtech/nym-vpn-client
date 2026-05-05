import { useAppStore } from '../../store/index';
import { Score } from '../../types/index';
import { ScoreIndicator } from '../node/ScoreIndicator';

export function ScoreIndicatorContainer({ score }: { score?: Score }) {
  const state = useAppStore((s) => s.state);
  const gatewaySelectionAlgorithmConfig = useAppStore(
    (s) => s.gatewaySelectionAlgorithmConfig,
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
