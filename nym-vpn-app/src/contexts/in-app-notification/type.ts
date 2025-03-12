import { ToastProps } from '../../ui';

export type Notification = Omit<
  ToastProps,
  'open' | 'onOpenChange' | 'defaultOpen'
> & {
  id?: string;
  onClose?: () => void;
  // Number of second to wait before allowing sending the same notification again\
  // Note: this requires the `id` field to be set
  throttle?: number;
};

export type NotificationCtxState = {
  // Currently displayed notification
  readonly current: Notification | null;
  // To be called when the current notification is closed or finished
  onClose: () => void;
  // Adds a notification/s to the end of the stack
  push: (notification: Notification) => void;
  // Removes all notifications from the stack
  clear: () => void;
};
