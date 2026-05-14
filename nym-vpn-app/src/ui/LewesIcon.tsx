import { LewesIcon, LewesLightIcon } from '../assets';
import { useAppStore } from '../store';

export function LewesIconComponent() {
  const uiTheme = useAppStore((s) => s.uiTheme);

  return uiTheme === 'dark' ? <LewesIcon /> : <LewesLightIcon />;
}
