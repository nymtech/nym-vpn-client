import { useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import {
  CardSwitch,
  Link,
  PageAnim,
  RadioGroup,
  RadioGroupOption,
  SettingsMenuCardBig,
} from '../../../ui';
import { dispatch, useMainState } from '../../../store';
import { AmneziaWgUrl, DomainFrontingUrl, QuicUrl } from '../../../constants';
import { useToast } from '../../../hooks';
import { BridgeMode } from '../../../types';

function AntiCensorship() {
  const { bridgeMode, backendFlags, state, frontingMode } = useMainState();
  const { add } = useToast();

  const { t } = useTranslation('settings');

  const bridgeOptions = useMemo<RadioGroupOption<BridgeMode>[]>(
    () => [
      {
        key: 'off',
        label: t('anti-censorship.bridge-mode.options.off.label'),
        desc: t('anti-censorship.bridge-mode.options.off.desc'),
        descWrap: true,
      },
      {
        key: 'auto',
        label: t('anti-censorship.bridge-mode.options.auto.label'),
        desc: t('anti-censorship.bridge-mode.options.auto.desc'),
        descWrap: true,
      },
      {
        key: 'on',
        label: t('anti-censorship.bridge-mode.options.on.label'),
        desc: t('anti-censorship.bridge-mode.options.on.desc'),
        descWrap: true,
      },
    ],
    [t],
  );

  const onBridgeModeChange = async (mode: BridgeMode) => {
    try {
      await invoke('set_bridge_mode', { mode });
      dispatch({ type: 'set-bridge-mode', mode });
    } catch (err: unknown) {
      console.error('Failed to set bridge mode', err);
      add({
        id: `bridge-mode-${mode}`,
        title: t('anti-censorship.bridge-mode.error'),
        type: 'error',
      });
      return;
    }

    if (state == 'connected' || state == 'connecting') {
      add({
        id: `bridge-mode-${mode}`,
        title: t('anti-censorship.bridge-mode.snackbar'),
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
            <div className="flex w-full min-w-0 flex-col gap-1 px-5 pt-4">
              <p className="text-text-primary truncate text-base select-none">
                {t('anti-censorship.bridge-mode.label')}
              </p>
              <p className="text-status-warning text-sm select-none">
                {t('anti-censorship.bridge-mode.warning')}
              </p>
            </div>
          }
        >
          <div className="flex flex-col gap-4">
            <p className="text-text-secondary text-sm whitespace-pre-line">
              {t(
                backendFlags.bridgeTransports
                  ? 'anti-censorship.bridge-mode.content-transports'
                  : 'anti-censorship.bridge-mode.content',
              )}
            </p>
            <RadioGroup
              value={bridgeMode}
              options={bridgeOptions}
              onChange={onBridgeModeChange}
              data-testid="bridge-mode-radio-group"
            />
            <Link
              className="w-fit text-sm"
              text={t('anti-censorship.bridge-mode.link')}
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
