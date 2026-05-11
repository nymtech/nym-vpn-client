import i18n from 'i18next';
import {
  DaemonStatus,
  NetworkEnv,
  VpndInfo,
  VpndStatus,
  isVpndNonCompat,
  isVpndOk,
} from '../types';
import { kvGet, kvSet } from '../kvStore';
import { ToastAddData } from '../hooks';
import { dispatch } from '../store';

export type TauriReq<
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  Req extends (a: never, b?: never) => Promise<any>,
> = {
  name: string;
  request: () => ReturnType<Req>;
  onFulfilled: (value: Awaited<ReturnType<Req>>) => void;
};

// Fires a list of Tauri requests concurrently and handles the results
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export async function fireRequests(requests: TauriReq<any>[]) {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  const promises = await Promise.allSettled(requests.map((r) => r.request()));

  promises.forEach((res, index) => {
    if (res.status === 'rejected') {
      console.info(`command [${requests[index].name}] failed with error`);
    }
    if (res.status === 'fulfilled') {
      requests[index].onFulfilled(res.value as never);
    }
  });
}

export function daemonStatusUpdate(
  status: VpndStatus,
  add: (data: ToastAddData) => string,
  close: (id: string) => void,
) {
  console.log('daemonStatusUpdate', status);
  dispatch({
    type: 'set-daemon-status',
    status: vpndStatusToState(status),
  });
  const info = getVpndInfo(status);
  if (info) {
    dispatch({ type: 'set-daemon-info', info });
    close('daemon-not-connected');
  }
  if (isVpndNonCompat(status)) {
    add({
      id: 'daemon-no-compat',
      title: i18n.t('daemon-no-compat', {
        ns: 'notifications',
        version: status.nonCompat.current.version,
        required: status.nonCompat.requirement,
      }),
      type: 'warn',
    });
  }
  if (status === 'down') {
    add({
      id: 'daemon-not-connected',
      title: i18n.t('daemon-not-connected', {
        ns: 'notifications',
      }),
      type: 'error',
    });
  }
}

export async function networkEnvChanged(status: VpndStatus) {
  if (status === 'down' || status === 'authDenied') {
    return false;
  }
  const prevEnv = await kvGet<NetworkEnv>('last-network-env');
  const newEnv = getVpndInfo(status)?.network;
  const hasChanged = prevEnv !== newEnv;
  if (hasChanged) {
    console.info(`network env changed [${newEnv}]`);
    await kvSet('last-network-env', newEnv);
  }
  return hasChanged;
}

export function getVpndInfo(status: VpndStatus): VpndInfo | null {
  if (isVpndOk(status) && status.ok) {
    return status.ok;
  }
  if (isVpndNonCompat(status)) {
    return status.nonCompat.current;
  }
  return null;
}

function vpndStatusToState(status: VpndStatus): DaemonStatus {
  if (isVpndOk(status)) {
    return 'ok';
  }
  if (isVpndNonCompat(status)) {
    return 'non-compat';
  }
  if (status === 'authDenied') {
    return 'auth-denied';
  }
  return 'down';
}
