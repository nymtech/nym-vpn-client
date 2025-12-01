import clsx from 'clsx';
import { Channel, invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { type } from '@tauri-apps/plugin-os';
import { Button, ButtonText, Dialog, MsIcon, Progress } from '../../ui';
import { DownloadUpdateEvent, UpdateMetadata } from '../../types';

const updaterEnabled = window._APP.updaterEnabled;
const os = type();
let initialized = false;
let contentLength: bigint | number = 20_000_000; // default to 20MB

function UpdateDialog() {
  const [update, setUpdate] = useState<UpdateMetadata | null>(null);
  const [isOpen, setIsOpen] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [progress, setProgress] = useState<number>(0); // 0% - 100%
  const version = update?.version || 'unknown';

  const { t } = useTranslation('home');

  const fetchUpdate = async () => {
    try {
      const metadata = await invoke<UpdateMetadata | null>('fetch_update');
      if (metadata) {
        console.info(`app update is available: v${metadata.version}`);
        setUpdate(metadata);
        setIsOpen(true);
      }
    } catch {}
  };

  useEffect(() => {
    if (initialized || os !== 'windows' || !updaterEnabled) {
      return;
    }
    initialized = true;

    // wait for a bit before checking for updates to allow the UI to have fully loaded
    setTimeout(() => {
      fetchUpdate();
    }, 300);
  }, []);

  const handleClose = () => {
    if (isUpdating) {
      // prevent the user from closing the dialog while updating
      return;
    }
    setIsOpen(false);
  };

  const onProgress = (event: DownloadUpdateEvent) => {
    console.log(`update download event`, event);
    switch (event.event) {
      case 'started':
        contentLength = event.data.contentLength;
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
    if (isUpdating) {
      return;
    }
    setIsUpdating(true);
    const onEvent = new Channel<DownloadUpdateEvent>();
    onEvent.onmessage = onProgress;
    try {
      await invoke('install_update', { onEvent });
    } catch {
      setIsOpen(false);
      setIsUpdating(false);
    }
  };

  if (!updaterEnabled) {
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
          className="text-xl text-baltic-sea dark:text-white text-center"
          data-testid="update-dialog-title"
        >
          {isUpdating
            ? t('app-update-progress.title', {
                version: version,
              })
            : t('app-update-available.title')}
        </DialogTitle>
      </div>
      {!isUpdating ? (
        <>
          <p
            className="text-iron dark:text-bombay text-center"
            data-testid="update-dialog-description"
          >
            {t('app-update-available.description', {
              version: version,
            })}
          </p>
          <p
            className="text-baltic-sea dark:text-white text-center max-w-2/3"
            data-testid="update-dialog-description"
          >
            {t('app-update-available.restart-note')}
          </p>
          <div className={clsx('flex flex-col items-center w-full gap-2')}>
            <Button onClick={onUpdate} className="mt-2" disabled={isUpdating}>
              <span className="text-lg text-black dark:text-baltic-sea">
                {t('app-update-available.button-update')}
              </span>
            </Button>
            <ButtonText
              onClick={() => {
                setIsOpen(false);
              }}
              className="mt-2"
              disabled={isUpdating}
              color="transparent"
            >
              {t('app-update-available.button-close')}
            </ButtonText>
          </div>
        </>
      ) : (
        <>
          <p
            className="text-iron dark:text-bombay text-center"
            data-testid="update-dialog-description"
          >
            {t('app-update-progress.description')}
          </p>
          <Progress
            value={progress}
            label={t('app-update-progress.bar-label')}
            className="w-full"
          />
          <p
            className="text-baltic-sea dark:text-white"
            data-testid="update-dialog-description"
          >
            {t('app-update-progress.restart-note')}
          </p>
        </>
      )}
    </Dialog>
  );
}

export default UpdateDialog;
