import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { LocationDetailsArticle } from '../../constants';
import { Button, Dialog, Link, MsIcon } from '../../ui';
import { capFirst } from '../../util';

export type Props = {
  isOpen: boolean;
  onClose: () => void;
};

function LocationDetailsDialog({ isOpen, onClose }: Props) {
  const { t } = useTranslation('nodeLocation');

  return (
    <Dialog
      open={isOpen}
      onClose={onClose}
      className="flex flex-col items-center gap-6"
    >
      <div className="flex flex-col items-center gap-4">
        <MsIcon
          icon="info"
          className="text-3xl text-baltic-sea dark:text-white"
        />
        <DialogTitle
          as="h3"
          className="text-xl text-baltic-sea dark:text-white text-center"
        >
          {t('location-details.title')}
        </DialogTitle>
      </div>

      <p className="text-center text-iron dark:text-bombay md:text-nowrap max-w-80">
        {t('location-details.description')}
      </p>

      <Link
        text={t('location-details.link')}
        url={LocationDetailsArticle}
        icon
      />

      <Button onClick={onClose} className="mt-2">
        <span className="text-lg text-black dark:text-baltic-sea">
          {capFirst(t('ok', { ns: 'glossary' }))}
        </span>
      </Button>
    </Dialog>
  );
}

export default LocationDetailsDialog;
