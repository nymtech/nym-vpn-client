import { useEffect, useState } from 'react';
import { motion } from 'motion/react';
import { useAppStore } from '../../store';
import { kvGet, kvSet } from '../../kvStore';
import {
  useDeviceLocationErrorToast,
  useGatewayIndependenceWatcher,
} from '../../hooks';
import DiagnosticsSuggestedDialog from './DiagnosticsSuggestedDialog';
import ExpiryReminderDialog from './ExpiryReminderDialog';
import GatewayIndependenceWarningDialog from './GatewayIndependenceWarningDialog';
import NetworkUpdateDialog from './NetworkUpdateDialog';
import UpdateDialog from './UpdateDialog';
import { NewBottomComponent } from './NewBottomComponent';
import { TunnelState } from './TunnelState';

const devMode = window._APP.devMode;
let compatChecked = false;

function Home() {
  const networkCompat = useAppStore((s) => s.networkCompat);
  const connected = useAppStore((s) => s.state === 'connected');
  useGatewayIndependenceWatcher();
  useDeviceLocationErrorToast();

  const [isDialogUpdateOpen, setIsDialogUpdateOpen] = useState(false);

  useEffect(() => {
    if (devMode || compatChecked) {
      return;
    }
    // A required update waits for an active connection to end, so Disconnect stays reachable.
    if (connected && networkCompat?.appUpdatePolicy === 'required') {
      return;
    }
    if (
      networkCompat &&
      (networkCompat.core === false || networkCompat.tauri === false)
    ) {
      compatChecked = true;
      // A dismissed prompt stays away until the minimum versions change.
      void kvGet<string>('app-update-dismissed').then((dismissed) => {
        if (
          networkCompat.appUpdatePolicy === 'required' ||
          dismissed !== networkCompat.floors
        ) {
          setIsDialogUpdateOpen(true);
        }
      });
    }
  }, [networkCompat, connected]);

  return (
    <>
      <UpdateDialog />
      <DiagnosticsSuggestedDialog />
      <GatewayIndependenceWarningDialog />
      <NetworkUpdateDialog
        isOpen={isDialogUpdateOpen}
        required={networkCompat?.appUpdatePolicy === 'required'}
        onClose={() => {
          setIsDialogUpdateOpen(false);
          void kvSet('app-update-dismissed', networkCompat?.floors);
        }}
        appUpdate={!networkCompat?.tauri}
        daemonUpdate={!networkCompat?.core}
      />
      <ExpiryReminderDialog />
      <motion.div
        initial={{ opacity: 0, x: '-1rem' }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.2, ease: 'easeOut' }}
        className="flex h-full flex-col"
      >
        <div className="flex min-h-0 grow flex-col items-center justify-center">
          <TunnelState />
        </div>
        <NewBottomComponent />
      </motion.div>
    </>
  );
}

export default Home;
