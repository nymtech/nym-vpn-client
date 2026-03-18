import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
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
            <span className="text-3xl text-malachite-moss dark:text-malachite leading-none">
              ·
            </span>
          )}
        </div>
      ))}
    </div>
  );
}

export function PincodeDialog({
  code,
  open,
  setOpen,
}: {
  code: string;
  open: boolean;
  setOpen: (open: boolean) => void;
}) {
  const { t } = useTranslation('account');
  const { copy } = useClipboard();

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
          <div className="flex items-center justify-center p-3 bg-malachite-moss/10 rounded-xl border border-malachite-moss">
            <MsIcon
              icon="lock"
              className="text-malachite-moss dark:text-malachite leading-none"
            />
          </div>
          <DialogTitle
            as="h3"
            className="text-xl text-baltic-sea dark:text-white text-center w-full truncate"
          >
            {t('autologin.title')}
          </DialogTitle>
          <p className="text-lg text-ash dark:text-white text-center">
            {t('autologin.description')}
          </p>
        </div>

        <PinCodeDigits code={code} />

        <Button className="w-full mt-3" onClick={() => copy(code)}>
          <div className="flex items-center justify-center gap-2">
            <MsIcon icon="content_copy" className="text-xl!" />
            <span>{t('autologin.copy-code')}</span>
          </div>
        </Button>
      </div>
    </Dialog>
  );
}
