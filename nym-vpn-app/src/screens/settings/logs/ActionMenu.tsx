import { useCallback, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { Menu } from '@base-ui-components/react';
import { invoke } from '@tauri-apps/api/core';
import { useInAppNotify, useMainState } from '../../../contexts';
import {
  ButtonIcon,
  ConfirmationDialog,
  ConfirmationDialogProps,
  MsIcon,
} from '../../../ui';

function Separator() {
  const { uiTheme } = useMainState();

  return (
    <Menu.Separator
      className={clsx(
        'h-px',
        uiTheme === 'light' && 'bg-faded-lavender',
        uiTheme === 'dark' && 'bg-iron',
      )}
    />
  );
}

function MenuItem({
  text,
  icon,
  onClick,
}: {
  text: string;
  icon: string;
  onClick: () => void;
}) {
  const { uiTheme } = useMainState();

  return (
    <Menu.Item
      onClick={onClick}
      className={clsx(
        'flex items-center gap-2 px-3 py-2 cursor-default select-none text-sm',
        'hover:bg-black/5 first:hover:rounded-t-sm last:hover:rounded-b-sm',
        uiTheme === 'light' && 'text-baltic-sea',
        uiTheme === 'dark' && 'text-white',
      )}
    >
      <MsIcon
        icon={icon}
        className={clsx(
          uiTheme === 'light' && 'text-iron',
          uiTheme === 'dark' && 'text-bombay',
        )}
      />
      {text}
    </Menu.Item>
  );
}

type DialogConfig = Omit<
  ConfirmationDialogProps,
  'isOpen' | 'isLoading' | 'onCancel'
>;

function ActionMenu() {
  const { t } = useTranslation('settings');

  const [activeDialog, setActiveDialog] = useState<
    keyof typeof dialogConfig | null
  >(null);
  const [isLoading, setIsLoading] = useState(false);

  const { uiTheme } = useMainState();
  const { push } = useInAppNotify();

  const handleDeleteLogs = useCallback(async () => {
    setIsLoading(true);
    try {
      console.log('deleting logs');
      await invoke('delete_logs');
      await invoke('delete_app_logs');
      push({
        message: t('logs.actions.delete.success'),
        type: 'info',
        duration: 3000,
        close: true,
      });
    } catch (error) {
      console.error('failed to delete logs', error);
      push({
        message: t('logs.actions.delete.error'),
        type: 'error',
        duration: 3000,
        close: true,
      });
    } finally {
      setActiveDialog(null);
      setIsLoading(false);
    }
  }, [push, t]);

  // eslint-disable-next-line @typescript-eslint/require-await
  const handleShareLogs = useCallback(async () => {
    // TODO: zip logs and open zip file in default file manager
    console.log('sharing logs');
  }, []);

  const dialogConfig = useMemo<Record<string, DialogConfig>>(
    () => ({
      delete: {
        icon: 'delete',
        title: t('logs.actions.delete.title'),
        description: t('logs.actions.delete.description'),
        confirmButtonText: t('logs.actions.delete.confirmButtonText'),
        confirmButtonColor: 'red',
        confirmButtonOutline: true,
        cancelButtonText: t('logs.actions.delete.cancelButtonText'),
        cancelButtonColor: 'gray',
        cancelButtonOutline: true,
        onConfirm: handleDeleteLogs,
      },
      share: {
        icon: 'share',
        title: t('logs.actions.share.title'),
        description: t('logs.actions.share.description'),
        confirmButtonText: t('logs.actions.share.confirmButtonText'),
        confirmButtonColor: 'red',
        confirmButtonOutline: true,
        cancelButtonText: t('logs.actions.share.cancelButtonText'),
        cancelButtonColor: 'gray',
        cancelButtonOutline: true,
        onConfirm: handleShareLogs,
      },
    }),
    [t, handleDeleteLogs, handleShareLogs],
  );

  return (
    <>
      <Menu.Root>
        <Menu.Trigger
          className={clsx(
            'flex items-center justify-center rounded-md mx-4 focus-visible:outline focus-visible:-outline-offset-1',
          )}
        >
          <ButtonIcon
            noDefaultSize
            color="chalk"
            icon="more_vert"
            onClick={() => {
              // Menu.Trigger handles the click
            }}
          />
        </Menu.Trigger>
        <Menu.Portal>
          <Menu.Positioner className="outline-none z-50" sideOffset={8}>
            <Menu.Popup
              className={clsx(
                'origin-(--transform-origin) rounded-md text-gray-900 shadow-lg shadow-gray-200 outline transition-[transform,scale,opacity] data-ending-style:scale-90 data-ending-style:opacity-0 data-starting-style:scale-90 data-starting-style:opacity-0',
                // dark theme
                uiTheme === 'dark' &&
                  'bg-charcoal shadow-none -outline-offset-1  text-white outline-iron',
                // light theme
                uiTheme === 'light' &&
                  'bg-white text-iron outline-faded-lavender',
              )}
            >
              <MenuItem
                icon="share"
                text={t('logs.actions.share.action-button')}
                onClick={() => {
                  setActiveDialog('share');
                }}
              />
              <Separator />
              <MenuItem
                icon="delete"
                text={t('logs.actions.delete.action-button')}
                onClick={() => {
                  setActiveDialog('delete');
                }}
              />
            </Menu.Popup>
          </Menu.Positioner>
        </Menu.Portal>
      </Menu.Root>
      {activeDialog && (
        <ConfirmationDialog
          icon={dialogConfig[activeDialog].icon}
          title={dialogConfig[activeDialog].title}
          description={dialogConfig[activeDialog].description}
          confirmButtonText={dialogConfig[activeDialog].confirmButtonText}
          confirmButtonColor={dialogConfig[activeDialog].confirmButtonColor}
          confirmButtonOutline={dialogConfig[activeDialog].confirmButtonOutline}
          cancelButtonColor={dialogConfig[activeDialog].cancelButtonColor}
          cancelButtonOutline={dialogConfig[activeDialog].cancelButtonOutline}
          cancelButtonText={dialogConfig[activeDialog].cancelButtonText}
          isOpen={!!activeDialog}
          isLoading={isLoading}
          onConfirm={dialogConfig[activeDialog].onConfirm}
          onCancel={() => setActiveDialog(null)}
        />
      )}
    </>
  );
}

export default ActionMenu;
