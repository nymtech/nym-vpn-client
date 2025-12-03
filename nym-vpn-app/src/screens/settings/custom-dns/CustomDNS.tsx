import { ReactNode, useState } from 'react';
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
import { useInAppNotify } from '../../../contexts/index';

function DefaultDnsServers() {
  const defaultDnsServers = [
    '192.0.2.44',
    '2001:db8::44',
    '198.51.100.44',
    '2001:db8::1337',
  ];

  return (
    <div className="flex flex-col">
      <p className="text-xs">Default DNS servers</p>
      <div className="py-3">
        {defaultDnsServers.map((dns) => (
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
          onDelete(item.dns);
        }}
        noDefaultSize
        className="shrink-0"
      />
    </div>
  );
}

const MAX_DNS_SERVERS = 5;

function CustomDns() {
  const { t } = useTranslation('settings');
  const [dnsList, setDnsList] = useState<DnsItem[]>([
    { id: '1.1.1.1', dns: '1.1.1.1' },
    { id: '1.0.0.1', dns: '1.0.0.1' },
  ]);
  const [inputValue, setInputValue] = useState('');
  const { push } = useInAppNotify();

  const handleAddDns = () => {
    const inputValueTrimmed = inputValue.trim();
    if (
      inputValueTrimmed === '' ||
      dnsList.length >= MAX_DNS_SERVERS ||
      dnsList.some((item) => item.dns === inputValueTrimmed)
    )
      return;
    setDnsList((prev) => [
      ...prev,
      { id: inputValueTrimmed, dns: inputValueTrimmed },
    ]);
    setInputValue('');
  };

  const handleApplyDns = () => {
    if (dnsList.length > 0) {
      push({
        message: 'Custom DNS applied. Reconnect to use it.',
        type: 'info',
      });
      return;
    }
  };

  const handleDeleteDns = (dns: string) => {
    setDnsList((prev) => prev.filter((d) => d.dns !== dns));
  };

  const handleReorder = (items: DnsItem[]) => {
    setDnsList(items);
  };

  return (
    <>
      <div className="flex flex-col">
        <p className="text-xs">
          Custom DNS servers ({dnsList.length}/{MAX_DNS_SERVERS})
        </p>
        <div className="py-3">
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
      {dnsList.length < MAX_DNS_SERVERS && (
        <div className="flex flex-row gap-2">
          <div className="flex-1">
            <TextInput
              placeholder={t('dns.details.input-placeholder')}
              onChange={setInputValue}
              value={inputValue}
              label={t('dns.details.input-label')}
              color="gray"
            />
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

      {dnsList.length > 0 && (
        <Button onClick={handleApplyDns} outline color="gray">
          <span className="text-lg text-black dark:text-baltic-sea">
            {t('dns.details.apply')}
          </span>
        </Button>
      )}
    </>
  );
}

function CustomDNS() {
  const { t } = useTranslation('settings');
  const [customDns, setCustomDns] = useState(true);

  const description = customDns
    ? t('dns.details.on.description')
    : t('dns.details.off.description');

  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6 select-none">
      <SettingsMenuCardBig
        header={
          <CardSwitch
            header={t('dns.details.title')}
            checked={customDns}
            onClick={() => {
              setCustomDns(!customDns);
            }}
          />
        }
      >
        <div className="flex flex-col gap-6">
          {/* Description */}
          <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
            {description}
          </p>

          {customDns ? <CustomDns /> : <DefaultDnsServers />}
        </div>
      </SettingsMenuCardBig>

      {customDns && (
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
