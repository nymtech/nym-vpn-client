import clsx from 'clsx';

type MsIconProps = {
  icon: string;
  style?: string;
};

function MsIcon({ icon, style }: MsIconProps) {
  return (
    <span className={clsx(['font-icon text-2xl select-none', style && style])}>
      {icon}
    </span>
  );
}

export default MsIcon;
