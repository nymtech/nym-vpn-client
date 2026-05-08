import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { ButtonNew, Dialog, MsIcon } from '../../../ui';
import { capFirst } from '../../../util';
import { NodeHop } from '../../../types';
import { Details } from './Details';

export type Props = {
  isOpen: boolean;
  onClose: () => void;
  node: NodeHop;
};

function LocationDetailsDialog({ isOpen, onClose, node }: Props) {
  const { t } = useTranslation('node-location');
  const title =
    node === 'entry'
      ? t('location-details.entry-title')
      : t('location-details.exit-title');

  return (
    <Dialog
      open={isOpen}
      onClose={onClose}
      className="flex flex-col items-center gap-6"
      data-testid="location-details-dialog"
    >
      <div className="flex flex-col items-center gap-4">
        <MsIcon
          icon="info"
          className="text-text-primary"
          data-testid="location-details-info-icon"
        />
        <DialogTitle
          as="h3"
          className="text-text-primary text-center text-xl"
          data-testid="location-details-title"
        >
          {title}
        </DialogTitle>
      </div>
      <Details node={node} />
      <ButtonNew
        onClick={onClose}
        className="mt-2"
        data-testid="location-details-close-button"
      >
        {capFirst(t('ok', { ns: 'glossary' }))}
      </ButtonNew>
    </Dialog>
  );
}

export default LocationDetailsDialog;
