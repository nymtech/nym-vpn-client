import {
  DarkBadIcon,
  DarkGoodIcon,
  DarkMediumIcon,
  DarkOfflineIcon,
  LightBadIcon,
  LightGoodIcon,
  LightMediumIcon,
  LightOfflineIcon,
} from '../../assets/icons';
import { useMainState } from '../../contexts';
import { Score } from '../../types';

export const ScoreIndicator = ({ score }: { score: Score }) => {
  const { uiTheme } = useMainState();

  switch (score) {
    case 'offline':
      return uiTheme === 'light' ? (
        <LightOfflineIcon className="h-6 w-6" />
      ) : (
        <DarkOfflineIcon className="h-6 w-6" />
      );
    case 'low':
      return uiTheme === 'light' ? (
        <LightBadIcon className="h-6 w-6" />
      ) : (
        <DarkBadIcon className="h-6 w-6" />
      );
    case 'medium':
      return uiTheme === 'light' ? (
        <LightMediumIcon className="h-6 w-6" />
      ) : (
        <DarkMediumIcon className="h-6 w-6" />
      );
    case 'high':
    default:
      return uiTheme === 'light' ? (
        <LightGoodIcon className="h-6 w-6" />
      ) : (
        <DarkGoodIcon className="h-6 w-6" />
      );
  }
};
