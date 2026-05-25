import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { invoke } from '@tauri-apps/api/core';
import { Button, MsIcon, PageAnim } from '../../../ui';
import { routes } from '../../../router';
import {
  dispatch,
  useAccountBackupConfirmed,
  useAccountLocallyGenerated,
} from '../../../store';
import { useClipboard, useIsLinux, useToast } from '../../../hooks';
import type { TAccountStateDetails } from '../../../types/tauri';

type View = 'idle' | 'prompting' | 'revealed';

export function RevealMnemonic() {
  const { t } = useTranslation('recovery-phrase');
  const navigate = useNavigate();
  const isLinux = useIsLinux();
  const isLocallyGenerated = useAccountLocallyGenerated();
  const isBackupConfirmed = useAccountBackupConfirmed();
  const { copy } = useClipboard();
  const { add } = useToast();

  const [view, setView] = useState<View>('idle');
  const [mnemonic, setMnemonic] = useState<string | undefined>(undefined);
  const [checked, setChecked] = useState(false);
  const [confirming, setConfirming] = useState(false);

  // Memory hygiene: drop mnemonic on unmount.
  useEffect(() => {
    return () => {
      setMnemonic(undefined);
    };
  }, []);

  // Non-Linux users should not reach this page.
  useEffect(() => {
    if (!isLinux) {
      navigate(routes.accountSettings, { replace: true });
    }
  }, [isLinux, navigate]);

  const handleReveal = async () => {
    setView('prompting');
    try {
      const phrase = await invoke<string>('get_stored_mnemonic');
      setMnemonic(phrase);
      setView('revealed');
    } catch (error: unknown) {
      console.warn('[RevealMnemonic] reveal denied or failed:', error);
      add({ title: t('auth-denied-toast'), type: 'error' });
      setView('idle');
    }
  };

  const handleCopy = async () => {
    if (!mnemonic) return;
    await copy(mnemonic);
  };

  const handleConfirm = async () => {
    setConfirming(true);
    try {
      await invoke('confirm_mnemonic_backup');
      // Re-fetch and dispatch the full details (the regular tunnel-event path
      // only carries the bare enum, so the flags wouldn't update otherwise).
      const details = await invoke<TAccountStateDetails>('get_account_state');
      dispatch({ type: 'set-account-state-details', details });
      navigate(routes.accountSettings);
    } catch (error) {
      console.error('[RevealMnemonic] confirm failed:', error);
    } finally {
      setConfirming(false);
    }
  };

  const handleBack = () => {
    setMnemonic(undefined);
    navigate(routes.accountSettings);
  };

  const showBackupCheckbox =
    isLocallyGenerated && !isBackupConfirmed && view === 'revealed';

  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-6 pb-2 select-none">
      <div className="border-warning text-warning bg-warning/10 flex items-center gap-3 rounded-lg border p-3">
        <MsIcon icon="report" />
        <p>{t('warning')}</p>
      </div>

      {view === 'idle' && (
        <Button onClick={handleReveal}>{t('reveal-button')}</Button>
      )}

      {view === 'prompting' && (
        <div className="text-text-secondary text-center">…</div>
      )}

      {view === 'revealed' && mnemonic && (
        <>
          <div className="grid grid-cols-3 gap-2">
            {mnemonic.split(/\s+/).map((word, i) => (
              <div
                key={`${i}-${word}`}
                className="bg-surface text-text-primary rounded p-2 text-center"
              >
                <span className="text-text-secondary mr-2 text-xs">
                  {i + 1}.
                </span>
                {word}
              </div>
            ))}
          </div>
          <Button onClick={handleCopy}>{t('copy-button')}</Button>

          {showBackupCheckbox && (
            <>
              <label className="flex cursor-pointer items-center gap-2">
                <input
                  type="checkbox"
                  checked={checked}
                  onChange={(e) => setChecked(e.target.checked)}
                  className="h-4 w-4"
                />
                <span className="text-text-primary text-sm">
                  {t('saved-checkbox')}
                </span>
              </label>
              <Button
                onClick={handleConfirm}
                disabled={!checked || confirming}
                loading={confirming}
              >
                {t('continue-button')}
              </Button>
            </>
          )}
        </>
      )}

      <Button onClick={handleBack} variant="outlined">
        {t('back-button')}
      </Button>
    </PageAnim>
  );
}
