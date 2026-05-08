import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useClipboard } from '../../hooks';
import { Button, ButtonIcon, Dialog, MsIcon } from '../../ui';

function PinCodeDigits({ code }: { code: string }) {
  const digits = code.split('');
  return (
    <div className="flex items-center justify-center gap-1">
      {digits.map((digit, i) => (
        <div key={i} className="flex items-center gap-2 font-mono">
          <span className="text-3xl font-bold text-black dark:text-white">
            {digit}
          </span>
          {i < digits.length - 1 && (
            <span className="text-primary text-3xl leading-none">·</span>
          )}
        </div>
      ))}
    </div>
  );
}

export function PincodeDialog({
  code,
  url,
  open,
  setOpen,
}: {
  code: string;
  url: string;
  open: boolean;
  setOpen: (open: boolean) => void;
}) {
  const { t } = useTranslation('account');
  const { copy } = useClipboard();

  const handleClick = async () => {
    await copy(code);
    openUrl(url);
  };

  return (
    <Dialog open={open} onClose={() => setOpen(false)}>
      <div className="flex flex-col items-center gap-6">
        <ButtonIcon
          className="self-end"
          color="chalk"
          icon="close"
          onClick={() => setOpen(false)}
        />

        <div className="flex flex-col items-center gap-3">
          <div className="bg-malachite-moss/10 border-malachite-moss flex items-center justify-center rounded-xl border p-3">
            <MsIcon icon="lock" className="text-primary leading-none" />
          </div>
          <DialogTitle
            as="h3"
            className="text-text-primary w-full truncate text-center text-xl"
          >
            {t('autologin.title')}
          </DialogTitle>
          <p className="text-ash text-center text-lg dark:text-white">
            {t('autologin.description')}
          </p>
        </div>

        <PinCodeDigits code={code} />

        <Button className="mt-3 w-full" onClick={handleClick}>
          <div className="flex items-center justify-center gap-2">
            <MsIcon icon="content_copy" className="text-xl!" />
            <span>{t('autologin.copy-code')}</span>
          </div>
        </Button>
      </div>
    </Dialog>
  );
}
