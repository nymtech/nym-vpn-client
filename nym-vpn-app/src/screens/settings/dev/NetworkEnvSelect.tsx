import { useState } from 'react';
import clsx from 'clsx';
import { Select } from '@headlessui/react';
import { motion } from 'motion/react';
import { invoke } from '@tauri-apps/api/core';
import { BackendError, NetworkEnv } from '../../../types';
import { MsIcon } from '../../../ui';

type NetworkOption = { value: NetworkEnv; label: string };

const options: NetworkOption[] = [
  { value: 'mainnet', label: 'Mainnet' },
  { value: 'canary', label: 'Canary' },
  { value: 'qa', label: 'QA' },
  { value: 'sandbox', label: 'Sandbox' },
];

export type Props = {
  current: NetworkEnv;
};

function NetworkEnvSelect({ current }: Props) {
  const [error, setError] = useState<string | null>();

  const handleOnSelect = async (network: NetworkEnv) => {
    setError(null);
    try {
      console.info('setting network to', network);
      await invoke<void>('set_network', { network });
    } catch (e: unknown) {
      const error = e as BackendError;
      console.warn('failed to set network', error);
      setError(`Failed to set network: ${error.key} - ${error.message}`);
    }
  };

  return (
    <div className="mt-2">
      <h3 className="text-lg text-baltic-sea dark:text-white font-medium">
        Network env
      </h3>
      <div className="flex flex-row flex-nowrap items-center text-sm">
        <MsIcon icon="priority_high" className="text-liquid-lava" />
        <p className="text-iron dark:text-bombay truncate">
          This require to restart the daemon to take effect
        </p>
      </div>
      <div className="relative">
        <Select
          className={clsx(
            'mt-3 block w-full appearance-none rounded-lg border-none',
            'bg-black/5 dark:bg-white/5 py-1.5 px-3 text-sm/6 text-black dark:text-white',
            'focus:outline-hidden data-focus:outline-2 data-focus:-outline-offset-2',
            'data-focus:outline-black/25 dark:data-focus:outline-white/25',
            // Make the text of each option black on Windows
            '*:text-black',
          )}
          defaultValue={current}
          onChange={(e) => {
            handleOnSelect(e.target.value as NetworkEnv);
          }}
        >
          {options.map(({ value, label }) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </Select>
        <MsIcon
          icon="keyboard_arrow_down"
          className="absolute right-2 top-1/2 transform -translate-y-1/2 text-black/50 dark:text-white/60"
        />
      </div>

      {error && (
        <motion.div
          initial={{ opacity: 0, x: -10 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.15, ease: 'easeInOut' }}
          className={clsx([
            'text-aphrodisiac overflow-y-scroll max-h-16 mt-3 break-words',
            'select-none',
          ])}
        >
          {error}
        </motion.div>
      )}
    </div>
  );
}

export default NetworkEnvSelect;
