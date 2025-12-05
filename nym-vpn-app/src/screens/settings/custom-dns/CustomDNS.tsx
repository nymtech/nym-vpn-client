import { useCallback, useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { useNavigate } from 'react-router';
import {
  CardSwitch,
  Link,
  PageAnim,
  SettingsMenuCard,
  SettingsMenuCardBig,
} from '../../../ui';
import { CustomDnsHelpUrl } from '../../../constants';
import useCustomDns from '../../../hooks/useCustomDns';
import { useInAppNotify, useMainState, useTopBar } from '../../../contexts';
import { routes } from '../../../router';
import { CustomDnsServers } from './CustomDnsServers';
import { DefaultDnsServers } from './DefaultDnsServers';
import { DnsItem } from './DnsItemContent';
import { ConfirmationDialog } from './ConfirmationDialog';

function CustomDNS() {
  const { t } = useTranslation('settings');
  const navigate = useNavigate();
  const {
    enabled: customDnsEnabled,
    toggle: toggleCustomDns,
    setCustomDns,
    customDns,
  } = useCustomDns();
  const { state } = useMainState();
  const { setCustomLeftNavHandler } = useTopBar();
  const { push } = useInAppNotify();

  const [dnsEnabledLocal, setDnsEnabledLocal] = useState(
    () => customDnsEnabled,
  );
  const [customDnsList, setCustomDnsList] = useState<DnsItem[]>(() =>
    customDns.map((dns) => ({ id: dns, dns })),
  );
  const [isConfirmationDialogOpen, setIsConfirmationDialogOpen] =
    useState(false);

  const hasUnsavedChanges = useMemo(() => {
    if (dnsEnabledLocal !== customDnsEnabled) return true;

    if (customDnsList.length !== customDns.length) return true;

    return !customDnsList.every((dns, index) => dns.dns === customDns[index]);
  }, [dnsEnabledLocal, customDnsEnabled, customDnsList, customDns]);

  const description = dnsEnabledLocal
    ? t('dns.details.on.description')
    : t('dns.details.off.description');

  const applyChanges = async () => {
    await toggleCustomDns(dnsEnabledLocal);
    await setCustomDns(customDnsList.map((item) => item.dns));
  };

  const discardChanges = () => {
    setIsConfirmationDialogOpen(false);
    setDnsEnabledLocal(customDnsEnabled);
    setCustomDnsList(customDns.map((dns) => ({ id: dns, dns })));
  };

  const handleDnsSwitchChange = async () => {
    const newState = !dnsEnabledLocal;
    // User can switch off immediately. Switching on will show a confirmation dialog before applying changes.
    if (newState === false) {
      await toggleCustomDns(false);
    }
    setDnsEnabledLocal(newState);
  };

  const handleApply = async () => {
    if (state === 'connected') {
      setIsConfirmationDialogOpen(true);
    } else {
      await applyChanges();
      push({
        message: t('dns.details.applied'),
        close: true,
        type: 'info',
      });
    }
  };

  const handleListChange = (dnsList: DnsItem[]) => {
    setCustomDnsList(dnsList);
  };

  const handleConfirmation = async () => {
    await applyChanges();
    navigate(routes.root);
  };

  const handleBackNavigation = useCallback(() => {
    if (hasUnsavedChanges && dnsEnabledLocal) {
      setIsConfirmationDialogOpen(true);
    } else {
      navigate(-1);
    }
  }, [hasUnsavedChanges, navigate, dnsEnabledLocal]);

  useEffect(() => {
    setCustomLeftNavHandler(handleBackNavigation);
    return () => {
      setCustomLeftNavHandler(null);
    };
  }, [handleBackNavigation, setCustomLeftNavHandler]);

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
            <CustomDnsServers
              onApplyDns={handleApply}
              customDnsList={customDnsList}
              onListChange={handleListChange}
            />
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
      <ConfirmationDialog
        isOpen={isConfirmationDialogOpen}
        onClose={discardChanges}
        onConfirm={handleConfirmation}
      />
    </PageAnim>
  );
}

export default CustomDNS;
