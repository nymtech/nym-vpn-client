import clsx from 'clsx';
import { Radio, RadioGroup } from '@headlessui/react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { CardSwitch, Link, PageAnim, SettingsMenuCardBig } from '../../../ui';
import { dispatch, useMainState } from '../../../store';
import { AmneziaWgUrl, DomainFrontingUrl, QuicUrl } from '../../../constants';
import { useToast } from '../../../hooks';
import { BridgeMode } from '../../../types';

const bridgeModes: BridgeMode[] = ['off', 'auto', 'on'];

// Segmented control: equal-width segments with a sliding thumb.
// Headless UI RadioGroup provides role=radiogroup/radio and arrow keys.
function BridgeModeSelector({
  value,
  onChange,
}: {
  value: BridgeMode;
  onChange: (mode: BridgeMode) => void;
}) {
  const { t } = useTranslation('settings');
  const index = Math.max(0, bridgeModes.indexOf(value));

  return (
    <RadioGroup
      value={value}
      onChange={onChange}
      aria-label={t('anti-censorship.bridge-mode.label')}
      className="bg-surface-sunken relative flex rounded-full p-1"
      data-testid="bridge-mode-radio-group"
    >
      <span
        aria-hidden
        style={{ '--i': index } as React.CSSProperties}
        className={clsx(
          'bg-brand-primary pointer-events-none absolute inset-y-1 start-1 w-[calc((100%-0.5rem)/3)] rounded-full shadow-sm',
          'translate-x-[calc(var(--i)*100%)] rtl:translate-x-[calc(var(--i)*-100%)]',
          'transition-transform duration-300 ease-out motion-reduce:transition-none',
        )}
      />
      {bridgeModes.map((mode) => (
        <Radio
          key={mode}
          value={mode}
          className={({ checked }) =>
            clsx(
              'relative z-10 flex min-h-10 flex-1 cursor-default items-center justify-center rounded-full text-sm',
              'focus-visible:ring-brand-primary focus:outline-hidden focus-visible:ring-2',
              checked ? 'text-brand-on-primary' : 'text-text-primary',
            )
          }
          data-testid={`bridge-mode-radio-group-option-${mode}`}
        >
          {t(`anti-censorship.bridge-mode.options.${mode}.label`)}
        </Radio>
      ))}
    </RadioGroup>
  );
}

function AntiCensorship() {
  const { bridgeMode, backendFlags, state, frontingMode } = useMainState();
  const { add } = useToast();

  const { t } = useTranslation('settings');

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
            <BridgeModeSelector
              value={bridgeMode}
              onChange={onBridgeModeChange}
            />
            <p className="text-text-primary text-sm">
              {t(`anti-censorship.bridge-mode.options.${bridgeMode}.desc`)}
            </p>
            <p className="text-text-secondary text-sm whitespace-pre-line">
              {t(
                backendFlags.bridgeTransports
                  ? 'anti-censorship.bridge-mode.content-transports'
                  : 'anti-censorship.bridge-mode.content',
              )}
            </p>
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
