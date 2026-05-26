import {
  SmileyIcon as SmileyIconAsset,
  SmileyLightIcon as SmileyLightIconAsset,
} from '../assets/icons';
import { useAppStore } from '../store';

function SmileyIcon({ className }: { className?: string }) {
  const uiTheme = useAppStore((s) => s.uiTheme);
  return uiTheme === 'dark' ? (
    <SmileyIconAsset className={className} />
  ) : (
    <SmileyLightIconAsset className={className} />
  );
}

export default SmileyIcon;
