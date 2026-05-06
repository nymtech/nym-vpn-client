import type { NavigateOptions, To } from 'react-router';
import { useNavigate } from 'react-router';
import { useCardAnimation } from '../contexts/CardAnimationContext';

export function useAnimatedNavigate() {
  const navigate = useNavigate();
  const { triggerExit } = useCardAnimation();

  return (to: To, options?: NavigateOptions): void => {
    void triggerExit().then(() => navigate(to, options));
  };
}
