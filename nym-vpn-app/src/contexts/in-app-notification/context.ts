import { createContext, useContext } from 'react';
import { NotificationCtxState } from './type';

const initialState: NotificationCtxState = {
  current: null,
  onClose: () => {
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
  createContext<NotificationCtxState>(initialState);
export const useInAppNotify = () => {
  return useContext(InAppNotificationContext);
};
