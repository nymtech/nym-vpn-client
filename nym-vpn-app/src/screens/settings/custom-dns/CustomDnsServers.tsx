import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button, TextInput } from '../../../ui';
import DraggableList from '../../../ui/DraggableList';
import { ipv4Regex, ipv6Regex } from '../../../utils';
import { DnsItem, DnsItemContent } from './DnsItemContent';

const MAX_DNS_SERVERS = 5;

export function CustomDnsServers({
  onApplyDns,
  customDnsList,
  onListChange,
}: {
  onApplyDns: (dnsList: string[]) => Promise<void>;
  customDnsList: DnsItem[];
  onListChange: (dnsList: DnsItem[]) => void;
}) {
  const { t } = useTranslation('settings');
  const [inputValue, setInputValue] = useState('');
  const [errorMessage, setErrorMessage] = useState('');
  const [isApplyingDns, setIsApplyingDns] = useState(false);

  const handleAddDns = () => {
    const inputValueTrimmed = inputValue.trim();
    const containsDuplicate = customDnsList.some(
      (item) => item.dns === inputValueTrimmed,
    );
    const isValid =
      ipv4Regex.test(inputValueTrimmed) || ipv6Regex.test(inputValueTrimmed);

    if (inputValueTrimmed === '') return;

    if (containsDuplicate) {
      setErrorMessage(t('dns.error.duplicate'));
      return;
    }

    if (!isValid) {
      setErrorMessage(t('dns.error.invalid'));
      return;
    }

    onListChange([
      ...customDnsList,
      { id: inputValueTrimmed, dns: inputValueTrimmed },
    ]);
    setInputValue('');
  };

  const applyDns = async () => {
    setIsApplyingDns(true);
    await onApplyDns(customDnsList.map((item) => item.dns));
    setIsApplyingDns(false);
  };

  const handleTextInputChange = (value: string) => {
    setInputValue(value);
    setErrorMessage('');
  };

  const handleDeleteDns = (dns: string) => {
    onListChange(customDnsList.filter((d) => d.id !== dns));
  };

  const handleReorder = (items: DnsItem[]) => {
    onListChange(items);
  };

  const handleApply = async () => {
    setIsApplyingDns(true);
    try {
      await applyDns();
    } finally {
      setIsApplyingDns(false);
    }
  };

  return (
    <>
      {customDnsList.length > 0 && (
        <div className="flex flex-col">
          <p className="text-xs">
            Custom DNS servers ({customDnsList.length}/{MAX_DNS_SERVERS})
          </p>
          <div className="my-3">
            <DraggableList
              items={customDnsList}
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
      {customDnsList.length < MAX_DNS_SERVERS && (
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
        disabled={customDnsList.length === 0 || isApplyingDns}
        onClick={handleApply}
        outline
        color="gray"
        spinner={isApplyingDns}
        className="group border border-iron dark:border-bombay hover:ring-0! dark:hover:ring-0!"
      >
        <span className="text-lg text-black group-hover:text-black/80 dark:text-white dark:group-hover:text-white/80">
          {t('dns.details.apply')}
        </span>
      </Button>
    </>
  );
}
