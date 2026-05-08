import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { ButtonNew, TextInput } from '../../../ui';
import DraggableList from '../../../ui/DraggableList';
import { ipv4Regex, ipv6Regex } from '../../../utils';
import { DnsItem, DnsItemContent } from './DnsItemContent';

const MAX_DNS_SERVERS = 5;

export function CustomDnsServers({
  hasUnsavedChanges,
  onApplyDns,
  customDnsList,
  onListChange,
}: {
  hasUnsavedChanges: boolean;
  onApplyDns: (dnsList: string[]) => Promise<void>;
  customDnsList: DnsItem[];
  onListChange: (dnsList: DnsItem[]) => void;
}) {
  const { t } = useTranslation('settings');

  const [inputValue, setInputValue] = useState('');
  const [errorMessage, setErrorMessage] = useState('');
  const [isApplyingDns, setIsApplyingDns] = useState(false);

  const isInputValueValid = useMemo(() => {
    const val = inputValue.trim();
    if (val === '0.0.0.0' || val === '255.255.255.255') return false;
    return ipv4Regex.test(val) || ipv6Regex.test(val);
  }, [inputValue]);

  const handleAddDns = () => {
    const inputValueTrimmed = inputValue.trim();
    if (inputValueTrimmed === '') return;

    const containsDuplicate = customDnsList.some(
      (item) => item.dns === inputValueTrimmed,
    );

    if (containsDuplicate) {
      setErrorMessage(t('dns.error.duplicate'));
      return;
    }

    if (!isInputValueValid) {
      setErrorMessage(t('dns.error.invalid'));
      return;
    }

    onListChange([
      ...customDnsList,
      { id: inputValueTrimmed, dns: inputValueTrimmed },
    ]);
    handleTextInputChange('');
  };

  const handleTextInputChange = (value: string) => {
    const inputValueTrimmed = value.trim();

    setInputValue(inputValueTrimmed);
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
      await onApplyDns(customDnsList.map((item) => item.dns));
    } finally {
      setIsApplyingDns(false);
    }
  };

  return (
    <>
      {customDnsList.length > 0 && (
        <div className="flex flex-col">
          <p className="text-xs">
            {t('dns.details.list-header')} ({customDnsList.length}/
            {MAX_DNS_SERVERS})
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
        <div className="flex flex-col gap-2">
          <div className="flex flex-row gap-2">
            <div className="h-full flex-1">
              <TextInput
                placeholder={t('dns.details.input-placeholder')}
                onChange={handleTextInputChange}
                value={inputValue}
                color="gray"
              />
            </div>
            <div className="h-full">
              <ButtonNew
                onClick={handleAddDns}
                variant="outlined"
                className="w-full"
              >
                {t('dns.details.add')}
              </ButtonNew>
            </div>
          </div>
          {errorMessage && (
            <p className="text-aphrodisiac text-xs">{errorMessage}</p>
          )}
        </div>
      )}

      <ButtonNew
        disabled={!hasUnsavedChanges || isApplyingDns}
        onClick={handleApply}
        variant="primary"
        loading={isApplyingDns}
      >
        <span>{t('dns.details.apply')}</span>
      </ButtonNew>
    </>
  );
}
