import { useEffect, useRef } from 'react';
import { useAppStore } from '../store';
import useI18nTunnelError from './useI18nTunnelError';
import useToast from './useToast';

function useDeviceLocationErrorToast() {
  const tunnelError = useAppStore((s) => s.tunnelError);
  const { tTE } = useI18nTunnelError();
  const { add } = useToast();
  // guard against re-triggering while a single error episode is handled
  const handledRef = useRef(false);

  useEffect(() => {
    if (tunnelError !== 'needs-device-location') {
      handledRef.current = false;
      return;
    }
    if (handledRef.current) {
      return;
    }
    handledRef.current = true;

    add({
      id: 'needs-device-location',
      title: tTE(tunnelError),
      type: 'error',
    });
  }, [tunnelError, tTE, add]);
}

export default useDeviceLocationErrorToast;
