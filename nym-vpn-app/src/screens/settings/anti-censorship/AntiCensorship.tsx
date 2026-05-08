import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { CardSwitch, Link, PageAnim, SettingsMenuCardBig } from '../../../ui';
import { dispatch, useMainState } from '../../../store';
import { AmneziaWgUrl, DomainFrontingUrl, QuicUrl } from '../../../constants';
import { useToast } from '../../../hooks';

function AntiCensorship() {
  const { quic, backendFlags, state, frontingMode } = useMainState();
  const { add } = useToast();

  const { t } = useTranslation('settings');

  const onQuicChange = async () => {
    const isChecked = !quic;
    try {
      await invoke('set_quic', { enabled: isChecked });
      dispatch({ type: 'set-quic', enabled: isChecked });
    } catch (err: unknown) {
      console.error('Failed to set QUIC', err);
      add({
        id: `quic-switch-${isChecked}`,
        title: t('anti-censorship.quic.error'),
        type: 'error',
      });
    }

    if (state == 'connected' || state == 'connecting') {
      add({
        id: `quic-switch-${isChecked}`,
        title: t(
          isChecked
            ? 'anti-censorship.snackbar-switch-on'
            : 'anti-censorship.snackbar-switch-off',
        ),
        type: 'info',
      });
    }
  };

  const handleFrontingModeChange = async () => {
    const value = frontingMode === 'onRetry' ? 'always' : 'onRetry';
    console.log('value', value);
    console.log('frontingMode', frontingMode);
    try {
      await invoke('set_fronting_mode', { mode: value });
      dispatch({ type: 'set-fronting-mode', mode: value });
    } catch (err: unknown) {
      console.error('Failed to set fronting mode', err);
      add({
        id: `fronting-mode-switch-${value}`,
        title: t('anti-censorship.stealth-api.error'),
        type: 'error',
      });
    }
  };

  if (!backendFlags.quic && !backendFlags.domainFronting) {
    return (
      <PageAnim className="mt-2 flex h-full flex-col gap-6 select-none">
        This feature is not available
      </PageAnim>
    );
  }

  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-6 select-none">
      <div className="text-text-secondary">{t('anti-censorship.intro')}</div>
      {backendFlags.quic && (
        <SettingsMenuCardBig
          header={
            <CardSwitch
              header={t('anti-censorship.quic.label')}
              subheader={t('anti-censorship.quic.warning')}
              subheaderColor="king-nacho"
              checked={quic}
              onClick={onQuicChange}
            />
          }
        >
          <div className="flex flex-col gap-2">
            <p className="text-text-secondary text-sm whitespace-pre-line">
              {t('anti-censorship.quic.content')}
            </p>
            <Link
              className="mt-2 w-fit text-sm"
              text={t('anti-censorship.quic.link')}
              url={QuicUrl}
              color="primary"
            />
          </div>
        </SettingsMenuCardBig>
      )}
      <SettingsMenuCardBig
        header={
          <CardSwitch
            header={t('anti-censorship.amneziawg.label')}
            checked={true}
            disabled={true}
            onClick={() => {
              /* TODO */
            }}
          />
        }
      >
        <div className="flex flex-col gap-2">
          <p className="text-text-secondary text-sm whitespace-pre-line">
            {t('anti-censorship.amneziawg.content')}
          </p>
          <Link
            className="mt-2 w-fit text-sm"
            text={t('anti-censorship.amneziawg.link')}
            url={AmneziaWgUrl}
            color="primary"
          />
        </div>
      </SettingsMenuCardBig>
      <SettingsMenuCardBig
        header={
          <CardSwitch
            header={t('anti-censorship.stealth-api.label')}
            checked={frontingMode === 'always'}
            onClick={handleFrontingModeChange}
          />
        }
      >
        <div className="flex flex-col gap-2">
          <p className="text-text-secondary text-sm whitespace-pre-line">
            {t('anti-censorship.stealth-api.content')}
          </p>
          <Link
            className="mt-2 w-fit text-sm"
            text={t('anti-censorship.stealth-api.link')}
            url={DomainFrontingUrl}
            color="primary"
          />
        </div>
      </SettingsMenuCardBig>
    </PageAnim>
  );
}

export default AntiCensorship;
