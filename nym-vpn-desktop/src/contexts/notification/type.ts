import { SnackbarProps } from '../../ui';

export type Notification = Omit<SnackbarProps, 'open' | 'onClose'> & {
  onClose?: () => void;
};

export type NotificationState = {
  // Notification list
  readonly stack: Notification[];
  // Currently displayed notification
  readonly current: Notification | null;
  // Moves to the next notification in the stack
  next: () => void;
  // Adds a notification to the end of the stack
  push: (notification: Notification) => void;
  // Removes all notifications from the stack
  clear: () => void;
};
