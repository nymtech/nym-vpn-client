import { ReactNode, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import {
  Button,
  ButtonIcon,
  CardSwitch,
  Link,
  MsIcon,
  PageAnim,
  SettingsMenuCard,
  SettingsMenuCardBig,
  TextInput,
} from '../../../ui';
import DraggableList, { DraggableListItem } from '../../../ui/DraggableList';
import { CustomDnsHelpUrl } from '../../../constants';
import useCustomDns from '../../../hooks/useCustomDns';
import { ipv4Regex, ipv6Regex } from '../../../utils';

function DefaultDnsServers({ defaultDnsList }: { defaultDnsList: string[] }) {
  return (
    <div className="flex flex-col">
      <p className="text-xs">Default DNS servers</p>
      <div className="py-3">
        {defaultDnsList.map((dns) => (
          <div
            key={dns}
            className="flex flex-row items-center gap-2 p-3 pl-0 border-t last:border-b border-bombay dark:border-iron"
          >
            <MsIcon icon="dns" className="text-iron dark:text-bombay" />
            <p className="text-base text-baltic-sea dark:text-white truncate">
              {dns}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
}

type DnsItem = DraggableListItem & {
  dns: string;
};

function DnsItemContent({
  item,
  dragHandle,
  onDelete,
}: {
  item: DnsItem;
  dragHandle: ReactNode;
  onDelete: (dns: string) => void;
}) {
  return (
    <div className="flex flex-row items-center justify-between gap-2 p-3 pl-">
      <div className="flex flex-row items-center gap-2 flex-1 min-w-0">
        {dragHandle}
        <p className="text-base text-baltic-sea dark:text-white truncate">
          {item.dns}
        </p>
      </div>
      <ButtonIcon
        icon="delete_outline"
        color="chalk"
        onClick={() => {
          onDelete(item.id);
        }}
        noDefaultSize
        className="shrink-0"
      />
    </div>
  );
}

const MAX_DNS_SERVERS = 5;

function CustomDns({
  onApplyDns,
  customDnsList,
}: {
  onApplyDns: (dnsList: string[]) => Promise<void>;
  customDnsList: string[];
}) {
  const { t } = useTranslation('settings');
  const [dnsList, setDnsList] = useState<DnsItem[]>(() =>
    customDnsList.map((dns) => ({ id: dns, dns: dns })),
  );
  const [inputValue, setInputValue] = useState('');
  const [errorMessage, setErrorMessage] = useState('');

  const handleAddDns = () => {
    const inputValueTrimmed = inputValue.trim();
    const containsDuplicate = dnsList.some(
      (item) => item.dns === inputValueTrimmed,
    );
    const isValid =
      ipv4Regex.test(inputValueTrimmed) || ipv6Regex.test(inputValueTrimmed);

    if (inputValueTrimmed === '') return;

    if (containsDuplicate) {
      setErrorMessage('Duplicate DNS address');
      return;
    }

    if (!isValid) {
      setErrorMessage('Invalid DNS address format');
      return;
    }

    setDnsList((prev) => [
      ...prev,
      { id: inputValueTrimmed, dns: inputValueTrimmed },
    ]);
    setInputValue('');
  };

  const handleApplyDns = () => {
    onApplyDns(dnsList.map((item) => item.dns));
  };

  const handleTextInputChange = (value: string) => {
    setInputValue(value);
    setErrorMessage('');
  };

  const handleDeleteDns = (dns: string) => {
    setDnsList((prev) => prev.filter((d) => d.id !== dns));
  };

  const handleReorder = (items: DnsItem[]) => {
    setDnsList(items);
  };

  return (
    <>
      {dnsList.length > 0 && (
        <div className="flex flex-col">
          <p className="text-xs">
            Custom DNS servers ({dnsList.length}/{MAX_DNS_SERVERS})
          </p>
          <div className="my-3">
            <DraggableList
              items={dnsList}
              onReorder={handleReorder}
              renderItem={(item, dragHandle) => (
                <DnsItemContent
                  item={item}
                  dragHandle={dragHandle}
                  onDelete={handleDeleteDns}
                />
              )}
            />
          </div>
        </div>
      )}
      {dnsList.length < MAX_DNS_SERVERS && (
        <div className="flex flex-row gap-2">
          <div className="flex-1 flex flex-col gap-2 ">
            <TextInput
              placeholder={t('dns.details.input-placeholder')}
              onChange={handleTextInputChange}
              value={inputValue}
              label={t('dns.details.input-label')}
              color="gray"
            />
            {errorMessage && (
              <p className="text-xs text-aphrodisiac">{errorMessage}</p>
            )}
          </div>
          <div className="shrink">
            <Button onClick={handleAddDns}>
              <span className="text-lg text-black dark:text-baltic-sea">
                {t('dns.details.add')}
              </span>
            </Button>
          </div>
        </div>
      )}

      <Button
        disabled={dnsList.length === 0}
        onClick={handleApplyDns}
        outline
        color="gray"
      >
        <span className="text-lg text-black dark:text-baltic-sea">
          {t('dns.details.apply')}
        </span>
      </Button>
    </>
  );
}

function CustomDNS() {
  const { t } = useTranslation('settings');
  const {
    enabled: customDnsEnabled,
    toggle: toggleCustomDns,
    customDns: customDnsList,
    setCustomDns,
    defaultDns: defaultDnsList,
  } = useCustomDns();

  const [dnsEnabledLocal, setDnsEnabledLocal] = useState(customDnsEnabled);

  const description = customDnsEnabled
    ? t('dns.details.on.description')
    : t('dns.details.off.description');

  const handleDnsSwitchChange = async () => {
    const newState = !dnsEnabledLocal;
    setDnsEnabledLocal(newState);
    if (newState === false) {
      await toggleCustomDns(false);
    }
  };

  const applyChanges = async (dnsList: string[]) => {
    console.log(
      '[applyChanges] dnsList',
      dnsList,
      'dnsEnabledLocal',
      dnsEnabledLocal,
    );
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
          {/* Description */}
          <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
            {description}
          </p>

          {dnsEnabledLocal ? (
            <CustomDns
              onApplyDns={applyChanges}
              customDnsList={customDnsList}
            />
          ) : (
            <DefaultDnsServers defaultDnsList={defaultDnsList} />
          )}
        </div>
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
      <Link
        className="w-fit text-sm"
        text={t('dns.details.link')}
        url={CustomDnsHelpUrl}
        color="primary"
        icon
      />
    </PageAnim>
  );
}

export default CustomDNS;
