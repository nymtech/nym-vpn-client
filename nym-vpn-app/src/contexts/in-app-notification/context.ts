import { createContext, useContext } from 'react';
import { NotificationState } from './type';

const initialState: NotificationState = {
  current: null,
  next: () => {
    /* SCARECROW */
  },
  push: () => {
    /* SCARECROW */
  },
  clear: () => {
    /* SCARECROW */
  },
};

export const InAppNotificationContext =
  createContext<NotificationState>(initialState);
export const useInAppNotify = () => {
  return useContext(InAppNotificationContext);
};
