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
  { value: 'evil', label: 'Evil' },
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
      setError(`Failed to set network: ${error.key} - ${error.message}`);
    }
  };

  return (
    <div className="mt-2" data-testid="network-env-select-container">
      <h3
        className="text-text-primary text-lg font-medium"
        data-testid="network-env-title"
      >
        Network env
      </h3>
      <div
        className="flex flex-row flex-nowrap items-center text-sm"
        data-testid="network-env-warning"
      >
        <MsIcon
          icon="priority_high"
          className="text-liquid-lava"
          data-testid="network-env-warning-icon"
        />
        <p className="text-text-secondary truncate">
          This require to restart the daemon to take effect
        </p>
      </div>
      <div className="relative" data-testid="network-env-select-wrapper">
        <Select
          className={clsx(
            'mt-3 block w-full appearance-none rounded-lg border-none',
            'bg-black/5 px-3 py-1.5 text-sm/6 text-black dark:bg-white/5 dark:text-white',
            'focus:outline-hidden data-focus:outline-2 data-focus:-outline-offset-2',
            'data-focus:outline-black/25 dark:data-focus:outline-white/25',
            // Make the text of each option black on Windows
            '*:text-black',
          )}
          defaultValue={current}
          onChange={(e) => {
            handleOnSelect(e.target.value as NetworkEnv);
          }}
          data-testid="network-env-select"
        >
          {options.map(({ value, label }) => (
            <option
              key={value}
              value={value}
              data-testid={`network-env-option-${value}`}
            >
              {label}
            </option>
          ))}
        </Select>
        <MsIcon
          icon="keyboard_arrow_down"
          className="absolute top-1/2 right-2 -translate-y-1/2 transform text-black/50 dark:text-white/60"
          data-testid="network-env-select-arrow"
        />
      </div>

      {error && (
        <motion.div
          initial={{ opacity: 0, x: -10 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.15, ease: 'easeInOut' }}
          className={clsx([
            'text-aphrodisiac mt-3 max-h-16 overflow-y-scroll break-words',
            'select-none',
          ])}
          data-testid="network-env-error"
        >
          {error}
        </motion.div>
      )}
    </div>
  );
}

export default NetworkEnvSelect;
