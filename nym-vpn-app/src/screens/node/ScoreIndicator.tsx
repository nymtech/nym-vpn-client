import {
  SignalBardBadIcon,
  SignalBardGoodIcon,
  SignalBardMediumIcon,
  SignalBardNoneIcon,
} from '../../assets/icons';
import { Score } from '../../types';

export const ScoreIndicator = ({ score }: { score?: Score }) => {
  switch (score) {
    case 'medium':
      return <SignalBardMediumIcon className="size-5" />;
    case 'low':
      return <SignalBardBadIcon className="size-5" />;
    case 'offline':
      return <SignalBardNoneIcon className="size-5" />;
    case 'high':
      return <SignalBardGoodIcon className="size-5" />;
    // Map an unknown/missing score to the no-signal icon, never a confident full
    // bar — an optimistic default is the silent-misleading bug this guards against.
    default:
      return <SignalBardNoneIcon className="size-5" />;
  }
};
