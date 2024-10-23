import { useState } from 'react';
import clsx from 'clsx';
import { Description, Field, Label, Select } from '@headlessui/react';
import { motion } from 'framer-motion';
import { invoke } from '@tauri-apps/api/core';
import { BackendError, NetworkEnv } from '../../../types';
import { Button, Dialog } from '../../../ui';

type NetworkOption = { value: NetworkEnv; label: string };

const options: NetworkOption[] = [
  { value: 'mainnet', label: 'Mainnet' },
  { value: 'cannary', label: 'Cannary' },
  { value: 'qa', label: 'QA' },
  { value: 'sandbox', label: 'Sandbox' },
];

export type Props = {
  open: boolean;
  onClose: () => void;
  current: NetworkEnv;
};

function NetworkEnvSelect({ open, onClose, current }: Props) {
  const [error, setError] = useState<string | null>();

  const handleOnSelect = async (network: NetworkEnv) => {
    setError(null);
    try {
      await invoke<void>('set_network', { network });
    } catch (e: unknown) {
      const error = e as BackendError;
      console.warn('failed to set network', error);
      setError(`Failed to set network: ${error.key} - ${error.message}`);
    }
  };

  return (
    <Dialog open={open} onClose={() => onClose()}>
      <Field>
        <Label className="text-lg text-baltic-sea dark:text-mercury-pinkish font-bold text-center">
          Network environment
        </Label>
        <Description className="text-sm/6 text-white/50">
          This require to restart the daemon to take effect
        </Description>
        <div className="relative">
          <Select
            className={clsx(
              'mt-3 block w-full appearance-none rounded-lg border-none',
              'bg-black/5 dark:bg-white/5 py-1.5 px-3 text-sm/6 text-black dark:text-white',
              'focus:outline-none data-[focus]:outline-2 data-[focus]:-outline-offset-2',
              'data-[focus]:outline-black/25 dark:data-[focus]:outline-white/25',
              // Make the text of each option black on Windows
              '*:text-black',
            )}
            defaultValue={current}
            onChange={(e) => {
              handleOnSelect(e.target.value as NetworkEnv);
            }}
            autoFocus
          >
            {options.map(({ value, label }) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </Select>
        </div>
      </Field>
      {error && (
        <motion.div
          initial={{ opacity: 0, x: -10 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.15, ease: 'easeInOut' }}
          className={clsx([
            'text-teaberry overflow-y-scroll max-h-16 mt-3 break-words',
            'select-none',
          ])}
        >
          {error}
        </motion.div>
      )}
      <Button onClick={onClose} className="mt-4 !py-1 !w-2/3">
        Ok
      </Button>
    </Dialog>
  );
}

export default NetworkEnvSelect;
