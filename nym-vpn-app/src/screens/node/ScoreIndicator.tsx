import {
  BadIcon,
  GoodIcon,
  LightBadIcon,
  LightGoodIcon,
  LightMediumIcon,
  LightOfflineIcon,
  MediumIcon,
  OfflineIcon,
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
        <OfflineIcon className="h-6 w-6" />
      );
    case 'low':
      return uiTheme === 'light' ? (
        <LightBadIcon className="h-6 w-6" />
      ) : (
        <BadIcon className="h-6 w-6" />
      );
    case 'medium':
      return uiTheme === 'light' ? (
        <LightMediumIcon className="h-6 w-6" />
      ) : (
        <MediumIcon className="h-6 w-6" />
      );
    case 'high':
    default:
      return uiTheme === 'light' ? (
        <LightGoodIcon className="h-6 w-6" />
      ) : (
        <GoodIcon className="h-6 w-6" />
      );
  }
};
