import { SnackbarProps } from '../../ui';

export type Notification = Omit<SnackbarProps, 'open' | 'onClose'> & {
  id?: string;
  onClose?: () => void;
  // Number of second to wait before allowing sending the same notification again\
  // Note: this requires the `id` field to be set
  throttle?: number;
};

export type NotificationState = {
  // Currently displayed notification
  readonly current: Notification | null;
  // Moves to the next notification in the stack
  next: () => void;
  // Adds a notification/s to the end of the stack
  push: (notification: Notification) => void;
  // Removes all notifications from the stack
  clear: () => void;
};
