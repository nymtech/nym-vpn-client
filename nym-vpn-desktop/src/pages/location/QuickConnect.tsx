import clsx from 'clsx';
import { useMainState } from '../../contexts';
import { QuickConnectPrefix } from '../../constants';

interface QuickConnectProps {
  onClick: (name: string, code: string) => void;
}

export default function QuickConnect({ onClick }: QuickConnectProps) {
  const { defaultNodeLocation } = useMainState();

  return (
    <div className="w-full py-5">
      <div
        role="presentation"
        className={clsx([
          'flex flex-row items-center w-full cursor-pointer',
          'hover:bg-gun-powder hover:bg-opacity-10',
          'dark:hover:bg-laughing-jack dark:hover:bg-opacity-10',
          'rounded-lg px-3 py-2',
        ])}
        onClick={() =>
          onClick(defaultNodeLocation.name, defaultNodeLocation.code)
        }
        onKeyDown={() =>
          onClick(defaultNodeLocation.name, defaultNodeLocation.code)
        }
      >
        <span className="font-icon text-2xl pl-1 pr-4 cursor-pointer">
          bolt
        </span>
        <div className="cursor-pointer text-base">{`${QuickConnectPrefix} (${defaultNodeLocation.name})`}</div>
      </div>
    </div>
  );
}
