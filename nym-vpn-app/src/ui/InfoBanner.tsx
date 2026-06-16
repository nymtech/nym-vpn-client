import clsx from 'clsx';
import MsIcon from './MsIcon';

type InfoBannerProps = {
  text: string;
  icon: string;
  variant: 'info' | 'warning' | 'error' | 'cornflower';
};

function InfoBanner({ text, icon, variant }: InfoBannerProps) {
  return (
    <div className="border-status-warning bg-status-warning/10 flex items-center gap-3 rounded-r-lg border-l-4 p-3">
      <MsIcon
        icon={icon}
        className={clsx('leading-none', {
          'text-status-warning': variant === 'warning',
          'text-status-error': variant === 'error',
          'text-status-cornflower': variant === 'cornflower',
          'text-status-info': variant === 'info',
        })}
      />
      <p className="text-status-warning text-sm">{text}</p>
    </div>
  );
}

export default InfoBanner;
