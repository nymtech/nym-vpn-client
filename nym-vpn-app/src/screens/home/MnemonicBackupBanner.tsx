import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { MsIcon } from '../../ui';
import { routes } from '../../router';
import {
  useAccountBackupConfirmed,
  useAccountLocallyGenerated,
  useAppStore,
} from '../../store';
import { useIsLinux } from '../../hooks';

export function MnemonicBackupBanner() {
  const { t } = useTranslation('home');
  const navigate = useNavigate();
  const isLinux = useIsLinux();
  const isLocallyGenerated = useAccountLocallyGenerated();
  const isBackupConfirmed = useAccountBackupConfirmed();
  const accountState = useAppStore((s) => s.accountState);

  const show =
    isLinux &&
    accountState === 'ready' &&
    isLocallyGenerated &&
    !isBackupConfirmed;

  if (!show) return null;

  return (
    <div className="border-cheddar bg-cheddar/10 text-cheddar mb-4 flex items-center gap-3 rounded-lg border p-3">
      <MsIcon icon="report" />
      <div className="flex-1">
        <p className="font-medium">{t('backup-banner.title')}</p>
        <p className="text-sm">{t('backup-banner.description')}</p>
      </div>
      <button
        type="button"
        onClick={() => navigate(routes.revealMnemonic)}
        className="text-cheddar hover:text-cheddar/80 shrink-0 cursor-default text-sm font-medium transition-colors"
      >
        {t('backup-banner.action')}
      </button>
    </div>
  );
}
