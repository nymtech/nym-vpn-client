import { Channel, invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { type } from '@tauri-apps/plugin-os';
import { Button, Dialog, MsIcon, Progress } from '../../../ui';
import {
  BackendError,
  DownloadUpdateEvent,
  UpdateMetadata,
} from '../../../types';

const updaterEnabled = window._APP.updaterEnabled;
let initialized = false;
const os = type();
let contentLength: bigint | number = 20_000_000; // default to 20MB

function UpdateDialog() {
  const [update, setUpdate] = useState<UpdateMetadata | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [progress, setProgress] = useState<number>(0); // 0% - 100%

  const { t } = useTranslation('home');

  const fetchUpdate = async () => {
    try {
      const metadata = await invoke<UpdateMetadata | null>('fetch_update');
      if (metadata) {
        console.info(`app update is available: v${metadata.version}`);
        setUpdate(metadata);
        setIsOpen(true);
      }
    } catch (error) {
      console.warn(
        'error checking for updates:',
        (error as BackendError).message,
      );
    }
  };

  useEffect(() => {
    if (initialized || os !== 'windows' || !updaterEnabled) {
      return;
    }
    initialized = true;

    fetchUpdate();
  }, []);

  const handleClose = () => {
    setIsOpen(false);
  };

  const onProgress = (event: DownloadUpdateEvent) => {
    console.log(`update download event`, event);
    switch (event.event) {
      case 'started':
        if (event.data.contentLength) {
          contentLength = event.data.contentLength;
        }
        break;
      case 'progress': {
        const chunkLength = event.data.chunkLength;
        setProgress((prev) => {
          const newProgress =
            prev + (chunkLength / Number(contentLength)) * 100;
          return Math.min(newProgress, 100);
        });
        break;
      }
      case 'finished':
        break;
    }
  };

  const onUpdate = async () => {
    setIsUpdating(true);
    const onEvent = new Channel<DownloadUpdateEvent>();
    onEvent.onmessage = onProgress;
    try {
      await invoke('install_update', { onEvent });
    } catch (error) {
      console.error('error during update:', (error as BackendError).message);
      setIsOpen(false);
      setIsUpdating(false);
    }
  };

  if (!updaterEnabled || !update) {
    return null;
  }

  return (
    <Dialog
      open={isOpen}
      onClose={handleClose}
      className="flex flex-col items-center gap-6"
      data-testid="update-dialog"
    >
      <div className="flex flex-col items-center gap-4">
        <MsIcon
          icon="info"
          className="text-3xl text-baltic-sea dark:text-white"
          data-testid="update-dialog-info-icon"
        />
        <DialogTitle
          as="h3"
          className="text-xl text-baltic-sea dark:text-white"
          data-testid="update-dialog-title"
        >
          {isUpdating
            ? t('app-update-progress.title', { version: update.version })
            : t('app-update-available.title')}
        </DialogTitle>
      </div>
      {!isUpdating ? (
        <>
          <p
            className="text-iron dark:text-bombay md:text-nowrap"
            data-testid="update-dialog-description"
          >
            {t('app-update-available.description', {
              version: update.version,
            })}
          </p>
          <p className="md:text-nowrap" data-testid="update-dialog-description">
            {t('app-update-available.note-close')}
          </p>
          <Button
            onClick={onUpdate}
            className="mt-2"
            data-testid="update-dialog-button"
            disabled={isUpdating}
          >
            <span className="text-lg text-black dark:text-baltic-sea">
              {t('app-update-available.button-update')}
            </span>
          </Button>
        </>
      ) : (
        <>
          <p
            className="text-iron dark:text-bombay md:text-nowrap"
            data-testid="update-dialog-description"
          >
            {t('app-update-progress.description')}
          </p>
          <Progress
            value={progress}
            label={t('app-update-progress.bar-label')}
          />
          <p
            className="text-iron dark:text-bombay md:text-nowrap"
            data-testid="update-dialog-description"
          >
            {t('app-update-progress.note-close')}
          </p>
        </>
      )}
    </Dialog>
  );
}

export default UpdateDialog;
