import { createContext, useContext } from 'react';
import { NotificationState } from './type';

export const initialState: NotificationState = {
  stack: [],
  current: null,
  next: () => {},
  push: () => {},
  clear: () => {},
};

export const NotificationContext =
  createContext<NotificationState>(initialState);
export const useNotifications = () => {
  return useContext(NotificationContext);
};
