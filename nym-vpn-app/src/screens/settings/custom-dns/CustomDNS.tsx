import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { CardSwitch, Link, PageAnim, SettingsMenuCardBig } from '../../../ui';
import { CustomDnsHelpUrl } from '../../../constants';
import useCustomDns from '../../../hooks/useCustomDns';
import { BackNavigationConfirmationDialog } from '../../../components';
import { useToast } from '../../../hooks';
import { CustomDnsServers } from './CustomDnsServers';
import { DefaultDnsServers } from './DefaultDnsServers';
import { DnsItem } from './DnsItemContent';

function CustomDNS() {
  const { t } = useTranslation('settings');
  const navigate = useNavigate();
  const {
    enabled: customDnsEnabled,
    toggle: toggleCustomDns,
    setCustomDns,
    customDns,
  } = useCustomDns();
  const { add } = useToast();

  const [customDnsList, setCustomDnsList] = useState<DnsItem[]>(() =>
    customDns.map((dns) => ({ id: dns, dns })),
  );

  const hasUnsavedChanges = useMemo(() => {
    if (customDnsList.length !== customDns.length) return true;

    return !customDnsList.every((dns, index) => dns.dns === customDns[index]);
  }, [customDnsList, customDns]);

  const applyChanges = async () => {
    if (customDnsList.length === 0) {
      await toggleCustomDns(false);
    }
    await setCustomDns(customDnsList.map((item) => item.dns));
    add({
      title: t('dns.details.applied'),
      type: 'info',
    });
  };

  const handleDnsSwitchChange = async () => {
    await toggleCustomDns(!customDnsEnabled);
  };

  const handleListChange = (dnsList: DnsItem[]) => {
    setCustomDnsList(dnsList);
  };

  const handleConfirmation = async () => {
    await applyChanges();
    navigate(-1);
  };

  const handleCancel = () => {
    navigate(-1);
  };

  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-6 select-none">
      <p className="text-text-secondary text-sm whitespace-pre-line">
        {t('dns.top-description')}
      </p>
      <DefaultDnsServers />

      <SettingsMenuCardBig
        header={
          <CardSwitch
            header={t('dns.details.title')}
            checked={customDnsEnabled}
            onClick={handleDnsSwitchChange}
            disabled={customDns.length === 0}
          />
        }
      >
        <div className="flex flex-col gap-6">
          <p className="text-text-secondary text-sm whitespace-pre-line">
            {t('dns.details.description')}
          </p>

          <CustomDnsServers
            hasUnsavedChanges={hasUnsavedChanges}
            onApplyDns={applyChanges}
            customDnsList={customDnsList}
            onListChange={handleListChange}
          />
        </div>
      </SettingsMenuCardBig>
      <Link
        className="w-fit text-sm"
        text={t('dns.details.link')}
        url={CustomDnsHelpUrl}
        color="primary"
        icon
      />

      <BackNavigationConfirmationDialog
        hasUnsavedChanges={hasUnsavedChanges}
        onConfirm={handleConfirmation}
        onCancel={handleCancel}
      />
    </PageAnim>
  );
}

export default CustomDNS;
