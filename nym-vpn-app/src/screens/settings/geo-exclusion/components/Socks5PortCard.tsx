import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  ButtonIconNew,
  CardDivider,
  CardNew,
  CardNewBody,
  CardNewHeader,
  TextInput,
} from '../../../../ui';
import { useClipboard, useDebounce } from '../../../../hooks';
import { isValidGeoExclusionPort } from '../utils/port';

type Socks5PortCardProps = {
  listenPort: number;
  onCommitPort: (port: number) => void;
};

function Socks5PortCard({ listenPort, onCommitPort }: Socks5PortCardProps) {
  const { t } = useTranslation('settings');
  const { copy } = useClipboard();

  const [draft, setDraft] = useState(String(listenPort));
  const [error, setError] = useState<string | null>(null);
  const commitPort = useDebounce(onCommitPort);

  useEffect(() => {
    setDraft(String(listenPort));
    setError(null);
  }, [listenPort]);

  const handleChange = (value: string) => {
    setDraft(value);

    if (!isValidGeoExclusionPort(value)) {
      commitPort.cancel();
      setError(t('geo-exclusion.port.invalid', { port: listenPort }));
      return;
    }

    setError(null);
    commitPort(Number(value));
  };

  return (
    <CardNew>
      <CardNewHeader>
        <div className="flex w-full items-center justify-between">
          <p className="text-text-secondary select-none">
            {t('geo-exclusion.port.server')}
          </p>
          <div className="flex items-center gap-2">
            <span className="text-text-primary font-mono">127.0.0.1</span>
            <ButtonIconNew
              icon="content_copy"
              onClick={() => copy('127.0.0.1', false)}
              clickFeedback
              size="small"
            />
          </div>
        </div>
      </CardNewHeader>
      <CardDivider className="" />
      <CardNewHeader>
        <div className="flex w-full items-center justify-between">
          <p className="text-text-secondary select-none">
            {t('geo-exclusion.port.socks5-port')}
          </p>
          <div className="flex items-center gap-2">
            <span className="text-text-primary font-mono">{listenPort}</span>
            <ButtonIconNew
              icon="content_copy"
              onClick={() => copy(String(listenPort), false)}
              clickFeedback
              size="small"
            />
          </div>
        </div>
      </CardNewHeader>
      <CardDivider className="" />
      <CardNewBody className="py-2">
        <div className="flex w-full flex-col gap-2 py-2">
          <p className="text-text-secondary text-sm select-none">
            {t('geo-exclusion.port.custom-port')}
          </p>
          <TextInput
            color="gray"
            value={draft}
            placeholder={String(listenPort)}
            onChange={handleChange}
          />
          {error ? (
            <p className="text-status-error text-xs">{error}</p>
          ) : (
            <p className="text-text-tertiary text-xs">
              {t('geo-exclusion.port.range')}
            </p>
          )}
        </div>
      </CardNewBody>
    </CardNew>
  );
}

export default Socks5PortCard;
