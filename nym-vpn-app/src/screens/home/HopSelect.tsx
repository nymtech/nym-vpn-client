import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { Country, Gateway, NodeHop, isGateway } from '../../types';
import { FlagIcon, MsIcon, countryCode } from '../../ui';
import { useLang } from '../../hooks';
import { useActionToast } from './util';

type HopSelectProps = {
  node: Country | Gateway;
  onClick: () => void;
  nodeHop: NodeHop;
  disabled?: boolean;
  locked?: boolean;
};

export default function HopSelect({
  nodeHop,
  node,
  onClick,
  disabled,
  locked,
}: HopSelectProps) {
  const { t } = useTranslation('home');
  const { getCountryName } = useLang();
  const toast = useActionToast('node-select');

  const handleClick = () => {
    if (disabled) {
      toast();
    } else {
      onClick();
    }
  };

  const SelectedCountry = (country: Country) => (
    <div className="flex flex-row items-center gap-3 overflow-hidden">
      <FlagIcon
        code={country.code.toLowerCase() as countryCode}
        alt={country.code}
      />
      <div
        className={clsx(['text-base truncate', disabled && 'cursor-default'])}
      >
        {getCountryName(country.code) || country.name}
      </div>
    </div>
  );

  const SelectedGateway = (gateway: Gateway) => (
    <div className="flex flex-row items-center gap-3 overflow-hidden">
      <FlagIcon
        code={gateway.country.code.toLowerCase() as countryCode}
        alt={gateway.country.code}
      />
      <div
        className={clsx(['text-base truncate', disabled && 'cursor-default'])}
      >
        {gateway.name}
      </div>
    </div>
  );

  return (
    <div
      className={clsx([
        'w-full flex flex-row justify-between items-center py-3 px-4',
        'text-baltic-sea dark:text-white',
        'border border-bombay dark:border-iron rounded-lg',
        !locked && [
          'hover:border-baltic-sea hover:ring-baltic-sea',
          'dark:hover:border-white dark:hover:ring-white',
        ],
        'relative transition select-none cursor-default',
        locked && 'opacity-50',
      ])}
      onKeyDown={handleClick}
      role="presentation"
      onClick={handleClick}
    >
      <div
        className={clsx([
          'absolute left-3 -top-2.5 px-1',
          'bg-faded-lavender dark:bg-ash text-xs',
          disabled && 'cursor-default',
        ])}
      >
        {nodeHop === 'entry' ? t('first-hop') : t('last-hop')}
      </div>
      {isGateway(node) ? SelectedGateway(node) : SelectedCountry(node)}
      <MsIcon icon="arrow_right" className="pointer-events-none" />
    </div>
  );
}
