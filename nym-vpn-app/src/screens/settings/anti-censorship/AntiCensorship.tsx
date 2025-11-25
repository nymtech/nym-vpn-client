import { useTranslation } from 'react-i18next';
import { CardSwitch, Link, PageAnim, SettingsMenuCardBig } from '../../../ui';
import {
  useInAppNotify,
  useMainDispatch,
  useMainState,
} from '../../../contexts';
import { StateDispatch } from '../../../types';
import { kvSet } from '../../../kvStore';
import { AmneziaWgUrl, DomainFrontingUrl, QuicUrl } from '../../../constants';

function AntiCensorship() {
  const { quic, backendFlags, state } = useMainState();
  const { push } = useInAppNotify();

  const dispatch = useMainDispatch() as StateDispatch;

  const { t } = useTranslation('settings');

  const onQuicChange = async () => {
    const isChecked = !quic;
    await kvSet('quic-enabled', isChecked);
    dispatch({ type: 'set-quic', enabled: isChecked });
    if (state == 'connected' || state == 'connecting') {
      push({
        id: `quic-switch-${isChecked}`,
        message: t(
          isChecked
            ? 'anti-censorship.snackbar-switch-on'
            : 'anti-censorship.snackbar-switch-off',
        ),
        throttle: 5,
        duration: 5000,
        close: true,
      });
    }
  };

  // const onDomainFrontingChange = async () => {
  //   const isChecked = !domainFronting;
  //   await kvSet('domain-fronting-enabled', isChecked);
  //   dispatch({ type: 'set-domain-fronting', enabled: isChecked });
  //   try {
  //     // TODO invoke command
  //   } catch {}
  // };

  if (!backendFlags.quic && !backendFlags.domainFronting) {
    return (
      <PageAnim className="h-full flex flex-col mt-2 gap-6 select-none">
        This feature is not available
      </PageAnim>
    );
  }

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6 select-none">
      <div className="text-iron dark:text-bombay">
        {t('anti-censorship.intro')}
      </div>
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
          <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
            {t('anti-censorship.amneziawg.content')}
          </p>
          <Link
            className="w-fit text-sm mt-2"
            text={t('anti-censorship.amneziawg.link')}
            url={AmneziaWgUrl}
            color="primary"
            icon
          />
        </div>
      </SettingsMenuCardBig>
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
            <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
              {t('anti-censorship.quic.content')}
            </p>
            <Link
              className="w-fit text-sm mt-2"
              text={t('anti-censorship.quic.link')}
              url={QuicUrl}
              color="primary"
              icon
            />
          </div>
        </SettingsMenuCardBig>
      )}
      {backendFlags.domainFronting && (
        <SettingsMenuCardBig
          header={
            <CardSwitch
              header={t('anti-censorship.stealth-api.label')}
              subheaderColor="king-nacho"
              // TODO keep it always ON for now
              checked={true}
              onClick={() => {
                /* TODO */
              }}
              disabled
            />
          }
        >
          <div className="flex flex-col gap-2">
            <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
              {t('anti-censorship.stealth-api.content')}
            </p>
            <Link
              className="w-fit text-sm mt-2"
              text={t('anti-censorship.stealth-api.link')}
              url={DomainFrontingUrl}
              color="primary"
              icon
            />
          </div>
        </SettingsMenuCardBig>
      )}
    </PageAnim>
  );
}

export default AntiCensorship;
