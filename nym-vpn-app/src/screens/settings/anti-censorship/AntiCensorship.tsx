import { useTranslation } from 'react-i18next';
import { CardSwitch, Link, PageAnim, SettingsMenuCardBig } from '../../../ui';
import { useMainDispatch, useMainState } from '../../../contexts';
import { StateDispatch } from '../../../types';
import { kvSet } from '../../../kvStore';

function AntiCensorship() {
  const { quic, domainFronting } = useMainState();

  const dispatch = useMainDispatch() as StateDispatch;

  const { t } = useTranslation('settings');

  const onQuicChange = async () => {
    const isChecked = !quic;
    await kvSet('quic-enabled', isChecked);
    dispatch({ type: 'set-quic', enabled: isChecked });
    try {
      // TODO invoke command
    } catch {}
  };

  const onDomainFrontingChange = async () => {
    const isChecked = !domainFronting;
    await kvSet('domain-fronting-enabled', isChecked);
    dispatch({ type: 'set-domain-fronting', enabled: isChecked });
    try {
      // TODO invoke command
    } catch {}
  };

  return (
    <PageAnim className="xs:max-w-lg h-full flex flex-col mt-2 gap-6 select-none">
      <div className="text-iron dark:text-bombay">
        {t('anti-censorship.intro')}
      </div>
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
          <p className="text-sm text-iron dark:text-bombay">
            {t('anti-censorship.quic.content')}
          </p>
          <Link
            className="w-fit text-sm text-iron dark:text-bombay"
            text={t('anti-censorship.quic.link')}
            // TODO add url
            url=""
            color="primary"
            icon
          />
        </div>
      </SettingsMenuCardBig>
      <SettingsMenuCardBig
        header={
          <CardSwitch
            header={t('anti-censorship.stealth-api.label')}
            subheader={t('anti-censorship.stealth-api.warning')}
            subheaderColor="king-nacho"
            checked={domainFronting}
            onClick={onDomainFrontingChange}
          />
        }
      >
        <div className="flex flex-col gap-2">
          <p className="text-sm text-iron dark:text-bombay">
            {t('anti-censorship.stealth-api.content')}
          </p>
          <Link
            className="w-fit text-sm text-iron dark:text-bombay"
            text={t('anti-censorship.stealth-api.link')}
            // TODO add url
            url=""
            color="primary"
            icon
          />
        </div>
      </SettingsMenuCardBig>
    </PageAnim>
  );
}

export default AntiCensorship;
