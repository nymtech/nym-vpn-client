export type Notification = {
  text: string;
  // Number of ms to wait before automatically close the snackbar
  autoHideDuration?: number;
  closeIcon?: boolean;
  clickAway?: boolean;
  position?: 'top' | 'bottom';
  onClose?: () => void;
};

export type NotificationState = {
  // Notification list
  readonly stack: Notification[];
  // Currently displayed notification
  readonly current: Notification | null;
  // Moves to the next notification in the stack
  next: () => void;
  // Adds a notification/s to the end of the stack
  push: (notification: Notification | Notification[]) => void;
  // Removes all notifications from the stack
  clear: () => void;
};
