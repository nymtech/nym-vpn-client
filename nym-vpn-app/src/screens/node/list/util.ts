import { Score } from '../../../types';

const scoreOrder: Record<Score, number> = {
  offline: 0,
  low: 1,
  medium: 2,
  high: 3,
};

export function sortByScore(a: Score, b: Score): number {
  if (a === b) {
    return 0;
  }
  return scoreOrder[b] - scoreOrder[a];
}
