import clsx from 'clsx';
import MsIcon from './MsIcon';

type InfoBannerProps = {
  text: string;
  icon: string;
  variant: 'info' | 'warning' | 'error';
};

function InfoBanner({ text, icon, variant }: InfoBannerProps) {
  return (
    <div
      className={clsx('flex items-center gap-3 rounded-r-lg border-l-4 p-3', {
        'border-status-warning bg-status-warning/10': variant === 'warning',
        'border-status-error bg-status-error/10': variant === 'error',
        'border-status-info bg-status-info/10': variant === 'info',
      })}
    >
      <MsIcon
        icon={icon}
        className={clsx('leading-none', {
          'text-status-warning': variant === 'warning',
          'text-status-error': variant === 'error',
          'text-status-info': variant === 'info',
        })}
      />
      <p
        className={clsx('text-sm', {
          'text-status-warning': variant === 'warning',
          'text-status-error': variant === 'error',
          'text-status-info': variant === 'info',
        })}
      >
        {text}
      </p>
    </div>
  );
}

export default InfoBanner;
