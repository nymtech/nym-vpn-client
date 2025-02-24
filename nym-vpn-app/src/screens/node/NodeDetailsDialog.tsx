import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import {
  Button,
  ButtonIcon,
  Dialog,
  FlagIcon,
  MsIcon,
  countryCode,
} from '../../ui';
import { capFirst } from '../../util';
import { UiCountry, UiGateway, useNodesState } from '../../contexts';
import { useClipboard, useLang } from '../../hooks';
import { getScoreIcon } from './list/util';

export type Props = {
  isOpen: boolean;
  onClose: () => void;
  ref: React.RefObject<UiGateway | UiCountry | null>;
};

function NodeDetailsDialog({ isOpen, onClose, ref }: Props) {
  const { t } = useTranslation('nodeLocation');
  const { vpnMode } = useNodesState();

  const gateway = ref.current as UiGateway;
  const { getCountryName } = useLang();
  const { copy } = useClipboard();

  if (!gateway) {
    return null;
  }
  const { country } = gateway;
  const scoreIcon = getScoreIcon(gateway, vpnMode);

  return (
    <Dialog
      open={isOpen}
      onClose={onClose}
      className="flex flex-col dark:text-white gap-8"
    >
      <h3 className="text-xl font-semibold">{gateway?.name}</h3>
      <div className="flex flex-row items-center gap-3">
        <MsIcon className={clsx(scoreIcon[1], 'text-xl')} icon={scoreIcon[0]} />
        <div className="w-[1px] bg-bombay dark:bg-dim-gray self-stretch" />
        <div className="flex flex-row items-center gap-2">
          <FlagIcon
            code={country.code.toLowerCase() as countryCode}
            alt={country.code}
            className="h-6"
          />
          <div>{getCountryName(country.code) || country.name}</div>
        </div>
      </div>
      <div className="flex flex-col gap-2">
        <p className="text-sm text-dim-gray dark:text-bombay">
          {t('node-details.id-label')}
        </p>
        <div className="flex flex-row">
          <div className="font-mono flex-wrap text-wrap break-words overflow-hidden max-w-72">
            {gateway.id}
          </div>
          <ButtonIcon
            icon="content_copy"
            onClick={() => copy(gateway.id, false)}
          />
        </div>
      </div>

      <Button onClick={onClose} className="mt-2">
        <span className="text-lg text-black dark:text-baltic-sea">
          {capFirst(t('ok', { ns: 'glossary' }))}
        </span>
      </Button>
    </Dialog>
  );
}

export default NodeDetailsDialog;
