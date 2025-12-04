import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import {
  CardSwitch,
  Link,
  PageAnim,
  SettingsMenuCard,
  SettingsMenuCardBig,
} from '../../../ui';
import { CustomDnsHelpUrl } from '../../../constants';
import useCustomDns from '../../../hooks/useCustomDns';
import { CustomDnsServers } from './CustomDnsServers';
import { DefaultDnsServers } from './DefaultDnsServers';

function CustomDNS() {
  const { t } = useTranslation('settings');
  const {
    enabled: customDnsEnabled,
    toggle: toggleCustomDns,
    setCustomDns,
  } = useCustomDns();

  const [dnsEnabledLocal, setDnsEnabledLocal] = useState(
    () => customDnsEnabled,
  );

  const description = dnsEnabledLocal
    ? t('dns.details.on.description')
    : t('dns.details.off.description');

  const handleDnsSwitchChange = async () => {
    const newState = !dnsEnabledLocal;
    // User can switch off immediately. Switching on will show a confirmation dialog before applying changes.
    if (newState === false) {
      await toggleCustomDns(false);
    }
    setDnsEnabledLocal(newState);
  };

  const applyChanges = async (dnsList: string[]) => {
    await toggleCustomDns(dnsEnabledLocal);
    await setCustomDns(dnsList);
  };

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6 select-none">
      <SettingsMenuCardBig
        header={
          <CardSwitch
            header={t('dns.details.title')}
            checked={dnsEnabledLocal}
            onClick={handleDnsSwitchChange}
          />
        }
      >
        <div className="flex flex-col gap-6">
          <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
            {description}
          </p>

          {dnsEnabledLocal ? (
            <CustomDnsServers onApplyDns={applyChanges} />
          ) : (
            <DefaultDnsServers />
          )}
        </div>
        <Link
          className="w-fit text-sm mt-5"
          text={t('dns.details.link')}
          url={CustomDnsHelpUrl}
          color="primary"
          icon
        />
      </SettingsMenuCardBig>

      {dnsEnabledLocal && (
        <motion.div
          initial={{ opacity: 0, translateY: -4 }}
          animate={{ opacity: 1, translateY: 0 }}
          transition={{ duration: 0.1, ease: 'easeIn' }}
        >
          <SettingsMenuCard
            title={t('dns.details.warning')}
            color="gray"
            noHoverEffect
          />
        </motion.div>
      )}
    </PageAnim>
  );
}

export default CustomDNS;
