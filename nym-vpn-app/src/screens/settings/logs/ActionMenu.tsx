import { useCallback, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { Menu } from '@base-ui-components/react';
import { invoke } from '@tauri-apps/api/core';
import {
  ConfirmationDialog,
  ConfirmationDialogProps,
  MsIcon,
} from '../../../ui';
import { useToast } from '../../../hooks';
import { useAppStore } from '../../../store';

function Separator() {
  const uiTheme = useAppStore((s) => s.uiTheme);

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
  const uiTheme = useAppStore((s) => s.uiTheme);

  return (
    <Menu.Item
      onClick={onClick}
      className={clsx(
        'flex cursor-default items-center gap-2 px-3 py-2 text-sm select-none',
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

  const uiTheme = useAppStore((s) => s.uiTheme);
  const { add } = useToast();

  const handleDeleteLogs = useCallback(async () => {
    setIsLoading(true);
    try {
      await invoke('delete_logs');
      await invoke('delete_app_logs');
      add({
        title: t('logs.actions.delete.success'),
        type: 'info',
      });
    } catch (error) {
      console.error('failed to delete logs', error);
      add({
        title: t('logs.actions.delete.error'),
        type: 'error',
      });
    } finally {
      setActiveDialog(null);
      setIsLoading(false);
    }
  }, [add, t]);

  const handleExportLogs = useCallback(async () => {
    setIsLoading(true);
    try {
      await invoke('zip_logs');
    } catch (error) {
      console.error('failed to zip logs', error);
      add({
        title: t('logs.actions.export.error'),
        type: 'error',
      });
    } finally {
      setActiveDialog(null);
      setIsLoading(false);
    }
  }, [add, t]);

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
        title: t('logs.actions.export.title'),
        description: t('logs.actions.export.description'),
        confirmButtonText: t('logs.actions.export.confirmButtonText'),
        confirmButtonColor: 'red',
        confirmButtonOutline: true,
        cancelButtonText: t('logs.actions.export.cancelButtonText'),
        cancelButtonColor: 'gray',
        cancelButtonOutline: true,
        onConfirm: handleExportLogs,
      },
    }),
    [t, handleDeleteLogs, handleExportLogs],
  );

  return (
    <>
      <Menu.Root>
        <Menu.Trigger
          className={clsx(
            'mx-4 flex items-center justify-center rounded-md',
            'focus-visible:outline focus-visible:-outline-offset-1',
            'hover:text-baltic-sea/70 dark:hover:text-white/80',
          )}
        >
          <MsIcon icon="more_vert" />
        </Menu.Trigger>
        <Menu.Portal>
          <Menu.Positioner className="z-50 outline-none" sideOffset={8}>
            <Menu.Popup
              className={clsx(
                'origin-(--transform-origin) rounded-md text-gray-900 shadow-lg shadow-gray-200 outline transition-[transform,scale,opacity] data-ending-style:scale-90 data-ending-style:opacity-0 data-starting-style:scale-90 data-starting-style:opacity-0',
                uiTheme === 'dark' &&
                  'bg-charcoal outline-iron text-white shadow-none -outline-offset-1',
                uiTheme === 'light' &&
                  'text-iron outline-faded-lavender bg-white',
              )}
            >
              <MenuItem
                icon="share"
                text={t('logs.actions.export.action-button')}
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
