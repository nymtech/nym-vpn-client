import { DialogTitle } from '@headlessui/react';
import { Trans, useTranslation } from 'react-i18next';
import { Button, Dialog, Link, MsIcon } from '../../ui';
import { capFirst } from '../../util';
import {
  LocationDetailsArticle,
  ResidentialIpServersUrl,
} from '../../constants';
import { NodeHop } from '../../types';

export type Props = {
  isOpen: boolean;
  onClose: () => void;
  node: NodeHop;
};

function LocationDetailsDialog({ isOpen, onClose, node }: Props) {
  const { t } = useTranslation('nodeLocation');
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
          className="text-3xl text-baltic-sea dark:text-white"
          data-testid="location-details-info-icon"
        />
        <DialogTitle
          as="h3"
          className="text-xl text-baltic-sea dark:text-white text-center"
          data-testid="location-details-title"
        >
          {title}
        </DialogTitle>
      </div>
      <div className="flex flex-col gap-2">
        <div className="flex flex-row items-center text-baltic-sea dark:text-white gap-2">
          <MsIcon icon="smart_display" />
          <h4 className="text-lg">{t('location-details.streaming.title')}</h4>
        </div>
        <p className="text-iron dark:text-bombay md:text-nowrap">
          <Trans
            i18nKey="location-details.streaming.description"
            ns="nodeLocation"
          >
            <Link
              url={ResidentialIpServersUrl}
              textClassName="underline text-black dark:text-white"
            >
              Residential IP servers
            </Link>
            optimized for streaming and content access. May experience slower
            speeds due to higher demand and hardware limitations.
          </Trans>
        </p>
      </div>
      <div className="flex flex-col gap-2">
        <div className="flex flex-row items-center text-baltic-sea dark:text-white gap-2">
          <MsIcon icon="location_on" />
          <h4 className="text-lg">{t('location-details.location.title')}</h4>
        </div>
        <p className="text-iron dark:text-bombay md:text-nowrap">
          <Trans
            i18nKey="location-details.location.description"
            ns="nodeLocation"
          >
            Displayed locations are
            <Link
              url={LocationDetailsArticle}
              textClassName="underline text-black dark:text-white"
            >
              determined from IP addresses
            </Link>
            and may not reflect exact physical locations.
          </Trans>
        </p>
      </div>
      <Button
        onClick={onClose}
        className="mt-2"
        data-testid="location-details-close-button"
      >
        <span className="text-lg text-black dark:text-baltic-sea">
          {capFirst(t('ok', { ns: 'glossary' }))}
        </span>
      </Button>
    </Dialog>
  );
}

export default LocationDetailsDialog;
