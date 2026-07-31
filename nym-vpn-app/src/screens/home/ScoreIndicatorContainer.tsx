import { Score } from '../../types/index';
import { ScoreIndicator } from '../node/ScoreIndicator';

export function ScoreIndicatorContainer({ score }: { score?: Score }) {
  return <ScoreIndicator score={score} />;
}
