import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Button, TextInput } from '../../../ui';
import DraggableList from '../../../ui/DraggableList';
import { ipv4Regex, ipv6Regex } from '../../../utils';
import { useInAppNotify, useMainState } from '../../../contexts/index';
import useCustomDns from '../../../hooks/useCustomDns';
import { DnsItem, DnsItemContent } from './DnsItemContent';
import { ConfirmationDialog } from './ConfirmationDialog';

const MAX_DNS_SERVERS = 5;

export function CustomDnsServers({
  onApplyDns,
}: {
  onApplyDns: (dnsList: string[]) => Promise<void>;
}) {
  const { state } = useMainState();
  const { t } = useTranslation('settings');
  const { push } = useInAppNotify();
  const { customDns: customDnsList } = useCustomDns();

  const [dnsList, setDnsList] = useState<DnsItem[]>(() =>
    customDnsList.map((dns) => ({ id: dns, dns: dns })),
  );
  const [inputValue, setInputValue] = useState('');
  const [errorMessage, setErrorMessage] = useState('');
  const [isConfirmationDialogOpen, setIsConfirmationDialogOpen] =
    useState(false);
  const [isApplyingDns, setIsApplyingDns] = useState(false);

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

  const applyDns = async () => {
    setIsApplyingDns(true);
    await onApplyDns(dnsList.map((item) => item.dns));
    setIsApplyingDns(false);
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

  const handleApplyButtonClick = async () => {
    if (state === 'connected') {
      setIsConfirmationDialogOpen(true);
    } else {
      await applyDns();
      push({
        message: 'DNS changes applied',
        close: true,
      });
    }
  };

  const handleHandleDialogConfirm = async () => {
    await applyDns();
    setIsConfirmationDialogOpen(false);
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
        disabled={dnsList.length === 0 || isApplyingDns}
        onClick={handleApplyButtonClick}
        outline
        color="gray"
        spinner={isApplyingDns}
      >
        <span className="text-lg text-black dark:text-baltic-sea">
          {t('dns.details.apply')}
        </span>
      </Button>
      <ConfirmationDialog
        isOpen={isConfirmationDialogOpen}
        onClose={() => setIsConfirmationDialogOpen(false)}
        onConfirm={handleHandleDialogConfirm}
      />
    </>
  );
}
