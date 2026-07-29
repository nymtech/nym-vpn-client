import clsx from 'clsx';
import { Radio, RadioGroup } from '@headlessui/react';
import { motion } from 'motion/react';
import { useTranslation } from 'react-i18next';
import { GatewayFastIcon } from '../../assets/icons/gateway-mode';
import { MixnetMark } from '../../assets/onboarding';

export type NetworkVariant = 'dvpn' | 'mixnet';

const VARIANTS = [
  { id: 'dvpn', Icon: GatewayFastIcon },
  { id: 'mixnet', Icon: MixnetMark },
] as const satisfies readonly { id: NetworkVariant; Icon: unknown }[];

type Props = {
  value: NetworkVariant;
  onChange: (value: NetworkVariant) => void;
};

export function VariantToggle({ value, onChange }: Props) {
  const { t } = useTranslation('onboarding');

  return (
    <RadioGroup
      value={value}
      onChange={onChange}
      aria-label={t('network.toggle.label')}
      className="bg-brand-primary flex w-full items-center rounded-full p-0.5"
    >
      {VARIANTS.map((variant) => (
        <Radio
          key={variant.id}
          value={variant.id}
          className={({ checked }) =>
            clsx(
              'relative flex flex-1 cursor-default items-center justify-center gap-1.5',
              'rounded-full px-3 py-2.5 text-sm font-bold transition-colors',
              'focus:outline-hidden',
              checked ? 'text-brand-primary' : 'text-brand-on-primary',
            )
          }
        >
          {({ checked }) => (
            <>
              {checked && (
                <motion.div
                  layoutId="onboarding-variant-pill"
                  className="bg-surface-bg border-brand-primary absolute inset-0 rounded-full border"
                  transition={{ duration: 0.3, ease: 'easeOut' }}
                />
              )}
              <variant.Icon className="z-10 h-4 w-auto" />
              <span className="z-10">{t(`network.toggle.${variant.id}`)}</span>
            </>
          )}
        </Radio>
      ))}
    </RadioGroup>
  );
}
