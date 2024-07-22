import { createContext, useContext } from 'react';
import { NotificationState } from './type';

const initialState: NotificationState = {
  stack: [],
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

export const NotificationContext =
  createContext<NotificationState>(initialState);
export const useNotifications = () => {
  return useContext(NotificationContext);
};
