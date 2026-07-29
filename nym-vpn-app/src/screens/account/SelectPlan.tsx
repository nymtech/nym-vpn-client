import { Trans, useTranslation } from 'react-i18next';
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button, MsIcon, PageAnim, Spinner } from '../../ui';
import { routes } from '../../router';
import { NymVpnTextLogo } from '../../assets';
import { useAutologin } from '../../contexts/autologin/context';
import { useDeepLink, useToast } from '../../hooks';
import { useAnimatedNavigate } from '../../hooks/useAnimatedNavigate';
import { DeeplinkTimeout } from '../../errors';
import { InteractiveCard } from '../home/InteractiveCard';
import { Planets } from './Planets';

function SelectPlan() {
  // Animated rather than plain `useNavigate`: `InteractiveCard` registers its
  // slide-down with `CardAnimationContext`, and only `useAnimatedNavigate`
  // triggers that before the route changes.
  const navigate = useAnimatedNavigate();
  const { t } = useTranslation('account');

  const [autologinLoading, setAutologinLoading] = useState(false);
  const { autologin, closeDialog } = useAutologin();
  const { startListening } = useDeepLink();
  const { add } = useToast();

  const handleClick = async () => {
    if (autologinLoading) return;

    setAutologinLoading(true);
    try {
      await autologin('autologinRenew');

      await startListening(600000);

      await invoke<void>('handle_subscription_payment');
      closeDialog();
      navigate(routes.root);
    } catch (error: unknown) {
      console.error('Select plan error: ', error);
      if (error instanceof DeeplinkTimeout) {
        add({
          title: t('autologin.timeout', { ns: 'errors' }),
          type: 'error',
        });
      }
    } finally {
      setAutologinLoading(false);
    }
  };

  return (
    <PageAnim className="relative flex h-full cursor-default flex-col items-center select-none">
      <Planets />

      {/* `relative` keeps the content above the glows without needing a z-index. */}
      <div className="relative flex grow flex-col items-center justify-center gap-6 px-4 text-center">
        <NymVpnTextLogo className="fill-text-primary h-13 w-48" />

        <h1 className="text-text-primary text-2xl">
          {t('select-a-plan.title')}
        </h1>

        <div className="flex flex-col gap-6">
          <p className="text-text-tertiary text-xs font-bold tracking-widest uppercase">
            {t('select-a-plan.favorite')}
          </p>
          <p className="text-text-primary text-sm font-bold whitespace-pre-line">
            <Trans
              i18nKey="select-a-plan.yearly"
              ns="account"
              components={{ green: <span className="text-brand-primary" /> }}
            />
          </p>
          <p className="text-text-secondary text-sm">
            <Trans
              i18nKey="select-a-plan.monthly"
              ns="account"
              components={{ bold: <strong /> }}
            />
          </p>
        </div>
      </div>

      {/* The wrapper is load-bearing: `InteractiveCard`'s own root is
          `flex h-full flex-col justify-end`, so as a direct child of this `h-full`
          column it would claim the whole height and squeeze the content above it.
          Nested in an auto-height block the percentage resolves to auto and the
          card collapses to its content — the same reason `NewBottomComponent`
          wraps it. It also supplies `bg-surface-elev rounded-2xl p-5` itself. */}
      <div className="relative w-full shrink-0">
        <InteractiveCard>
          <div className="flex flex-col items-center gap-8">
            <NymVpnTextLogo className="fill-text-primary h-[27px] w-[100px]" />
            <Button onClick={handleClick} variant="primary">
              <span className="flex items-center gap-2">
                {t('select-a-plan.button')}{' '}
                {autologinLoading ? <Spinner /> : <MsIcon icon="open_in_new" />}
              </span>
            </Button>
          </div>
        </InteractiveCard>
      </div>
    </PageAnim>
  );
}

export default SelectPlan;
