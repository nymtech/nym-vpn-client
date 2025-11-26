import { useState } from 'react';
import { useTranslation } from 'react-i18next';
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
import { CustomDnsHelpUrl } from '../../../constants';

function CustomDNS() {
  const { t } = useTranslation('settings');
  const [customDns, setCustomDns] = useState(false);
  const [inputValue, setInputValue] = useState('');
  const [dnsList, setDnsList] = useState<string[]>(['1.1.1.1', '1.0.0.1']);

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
        <div className="flex flex-col gap-2">
          <p className="text-sm text-iron dark:text-bombay whitespace-pre-line">
            {t('dns.details.description')}
          </p>
        </div>

        <div className="mt-5">
          <div className="flex flex-col mb-4">
            {dnsList.length > 0 && (
              <div className="border-t border-b bg-white dark:bg-charcoal overflow-hidden">
                {dnsList.map((dns, index) => (
                  <div key={index}>
                    <div className="flex flex-row items-center justify-between px-4 py-3">
                      <div className="flex flex-row items-center gap-3 flex-1 min-w-0">
                        <MsIcon
                          icon="dns"
                          className="text-iron dark:text-bombay shrink-0"
                        />
                        <span className="text-base text-baltic-sea dark:text-white truncate">
                          {dns}
                        </span>
                      </div>
                      <ButtonIcon
                        icon="delete_outline"
                        color="chalk"
                        onClick={() => {
                          setDnsList((prev) =>
                            prev.filter((_, i) => i !== index),
                          );
                        }}
                        noDefaultSize
                        className="shrink-0"
                      />
                    </div>
                    {index < dnsList.length - 1 && (
                      <div className="h-px bg-mercury dark:bg-mine-shaft" />
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>

          <div className="flex flex-row gap-4">
            <div className="flex-1">
              <TextInput
                placeholder="Default DNS: 1.1.1.1"
                onChange={setInputValue}
                value={inputValue}
                label={t('dns.details.input-label')}
              />
            </div>
            <div className="shrink">
              <Button
                onClick={() => {
                  setDnsList((prev) => [...prev, inputValue]);
                  setInputValue('');
                }}
              >
                <span className="text-lg text-black dark:text-baltic-sea">
                  {t('dns.details.add')}
                </span>
              </Button>
            </div>
          </div>
        </div>
      </SettingsMenuCardBig>
      <SettingsMenuCard
        title={t('dns.details.warning')}
        noHoverEffect
        className="dark:bg-mine-shaft!"
      />
      <Link
        className="w-fit text-sm mt-2"
        text={t('dns.details.link')}
        url={CustomDnsHelpUrl}
        color="primary"
        icon
      />
    </PageAnim>
  );
}

export default CustomDNS;
