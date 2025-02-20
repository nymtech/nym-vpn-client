import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { Country, Gateway, NodeHop, isGateway } from '../../types';
import { useInAppNotify, useMainState } from '../../contexts';
import { FlagIcon, MsIcon, countryCode } from '../../ui';
import { useLang, useThrottle } from '../../hooks';
import { HomeThrottleDelay } from '../../constants';

type HopSelectProps = {
  node: Country | Gateway;
  onClick: () => void;
  nodeHop: NodeHop;
  disabled?: boolean;
};

export default function HopSelect({
  nodeHop,
  node,
  onClick,
  disabled,
}: HopSelectProps) {
  const { state, daemonStatus } = useMainState();
  const { t } = useTranslation('home');
  const { push } = useInAppNotify();
  const { getCountryName } = useLang();

  const showSnackbar = useThrottle(
    () => {
      let text = '';
      switch (state) {
        case 'Connected':
          text = t('snackbar-disabled-message.connected');
          break;
        case 'Connecting':
          text = t('snackbar-disabled-message.connecting');
          break;
        case 'Disconnecting':
          text = t('snackbar-disabled-message.disconnecting');
          break;
        case 'Offline':
        case 'OfflineAutoReconnect':
          text = t('snackbar-disabled-message.offline');
          break;
      }
      if (daemonStatus === 'NotOk') {
        text = t('snackbar-disabled-message.daemon-not-connected');
      }
      if (text.length > 0) {
        push({
          text,
          position: 'top',
        });
      }
    },
    HomeThrottleDelay,
    [state, daemonStatus],
  );

  const handleClick = () => {
    if (disabled) {
      showSnackbar();
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
        'text-baltic-sea dark:text-mercury-pinkish',
        'border border-cement-feet dark:border-gun-powder rounded-lg',
        'hover:border-baltic-sea hover:ring-baltic-sea',
        'dark:hover:border-mercury-pinkish dark:hover:ring-mercury-pinkish',
        'relative transition select-none cursor-default',
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
