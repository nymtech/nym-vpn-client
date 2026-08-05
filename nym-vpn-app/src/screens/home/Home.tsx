import { useEffect, useState } from 'react';
import { type } from '@tauri-apps/plugin-os';

import { motion } from 'motion/react';
import { useAppStore } from '../../store';
import { useGatewayIndependenceWatcher } from '../../hooks';
import DiagnosticsSuggestedDialog from './DiagnosticsSuggestedDialog';
import GatewayIndependenceWarningDialog from './GatewayIndependenceWarningDialog';
import NetworkUpdateDialog from './NetworkUpdateDialog';
import UpdateDialog from './UpdateDialog';
import { NewBottomComponent } from './NewBottomComponent';
import { TunnelState } from './TunnelState';

const devMode = window._APP.devMode;
const os = type();
let compatChecked = false;

function Home() {
  const networkCompat = useAppStore((s) => s.networkCompat);
  useGatewayIndependenceWatcher();

  const [isDialogUpdateOpen, setIsDialogUpdateOpen] = useState(false);

  useEffect(() => {
    if (devMode || compatChecked) {
      return;
    }
    if (
      networkCompat &&
      (networkCompat.core === false || networkCompat.tauri === false)
    ) {
      // if either core or tauri is not compatible, show the update dialog
      compatChecked = true;
      setIsDialogUpdateOpen(true);
    }
  }, [networkCompat]);

  return (
    <>
      <UpdateDialog />
      <DiagnosticsSuggestedDialog />
      <GatewayIndependenceWarningDialog />
      {os !== 'windows' && (
        <NetworkUpdateDialog
          isOpen={isDialogUpdateOpen}
          onClose={() => setIsDialogUpdateOpen(false)}
          appUpdate={!networkCompat?.tauri}
          daemonUpdate={!networkCompat?.core}
        />
      )}
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
