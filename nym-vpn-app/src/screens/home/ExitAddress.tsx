import { useTranslation } from 'react-i18next';
import { useAppStore } from '../../store';
import { TunnelData, isWireguardData } from '../../types';

// The exit gateway's public address: what websites see. It is shared by
// everyone on that exit and is never the user's own address, so it is
// labelled that way. Screen readers get one sentence in reading order (not a
// live region, so it is never pushed as an update).

// "1.2.3.4:51820" -> "1.2.3.4", "[2001:db8::1]:51820" -> "2001:db8::1"
export function hostOf(endpoint: string): string {
  const v6 = /^\[([^\]]+)\](?::\d+)?$/.exec(endpoint);
  if (v6) return v6[1];
  const v4 = /^([^:]+):\d+$/.exec(endpoint);
  return v4 ? v4[1] : endpoint;
}

export function exitAddressOf(data: TunnelData): string | null {
  const ip = isWireguardData(data) ? hostOf(data.exit.endpoint) : data.exitIp;
  return ip.trim() ? ip : null;
}

function ExitAddress() {
  const state = useAppStore((s) => s.state);
  const tunnel = useAppStore((s) => s.tunnel);
  const { t } = useTranslation('home');

  const ip = tunnel ? exitAddressOf(tunnel.data) : null;
  if (state !== 'connected' || !ip) {
    return null;
  }

  return (
    <div
      className="cursor-default text-center select-text"
      data-testid="exit-address"
    >
      <span className="sr-only">{t('exit-address.aria', { ip })}</span>
      <p className="text-text-primary text-base" aria-hidden="true">
        {t('exit-address.label')} · <span className="tabular-nums">{ip}</span>
        <span className="text-text-tertiary"> · {t('exit-address.note')}</span>
      </p>
    </div>
  );
}

export default ExitAddress;
