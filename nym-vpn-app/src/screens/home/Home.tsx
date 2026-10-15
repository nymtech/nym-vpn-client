import { useEffect, useState } from 'react';
import { motion } from 'motion/react';
import { useAppStore } from '../../store';
import {
  useDeviceLocationErrorToast,
  useGatewayIndependenceWatcher,
} from '../../hooks';
import DiagnosticsSuggestedDialog from './DiagnosticsSuggestedDialog';
import GatewayIndependenceWarningDialog from './GatewayIndependenceWarningDialog';
import NetworkUpdateDialog from './NetworkUpdateDialog';
import UpdateDialog from './UpdateDialog';
import useExpiryReminderToast from './useExpiryReminderToast';
import { NewBottomComponent } from './NewBottomComponent';
import { TunnelState } from './TunnelState';

const devMode = window._APP.devMode;
let compatChecked = false;

function Home() {
  const networkCompat = useAppStore((s) => s.networkCompat);
  const connected = useAppStore((s) => s.state === 'connected');
  useGatewayIndependenceWatcher();
  useDeviceLocationErrorToast();
  useExpiryReminderToast();

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
      setIsDialogUpdateOpen(true);
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
        onClose={() => setIsDialogUpdateOpen(false)}
        appUpdate={!networkCompat?.tauri}
        daemonUpdate={!networkCompat?.core}
      />
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
