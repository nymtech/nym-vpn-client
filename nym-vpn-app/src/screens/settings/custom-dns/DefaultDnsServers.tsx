import { useState } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import { useTranslation } from 'react-i18next';
import useCustomDns from '../../../hooks/useCustomDns';
import { ButtonText } from '../../../ui';

export function DefaultDnsServers() {
  const { t } = useTranslation('settings');
  const { defaultDns } = useCustomDns();
  const [isDefaultDnsVisible, setIsDefaultDnsVisible] = useState(false);

  const buttonText = isDefaultDnsVisible
    ? t('dns.hide-default-dns')
    : t('dns.view-default-dns');

  return (
    <div className="flex flex-col items-start justify-center">
      <ButtonText
        color="transparent"
        className="text-text-secondary! px-0! text-sm"
        onClick={() => setIsDefaultDnsVisible((v) => !v)}
      >
        {buttonText}
      </ButtonText>

      <AnimatePresence initial={false}>
        {isDefaultDnsVisible && (
          <motion.div
            key="dns-list"
            initial={{ opacity: 0, translateY: -8, height: 0 }}
            animate={{ opacity: 1, translateY: 0, height: 'auto' }}
            exit={{ opacity: 0, translateY: -8, height: 0 }}
            transition={{ duration: 0.2, ease: 'easeInOut' }}
            style={{ overflow: 'hidden' }}
          >
            <ul className="py-3">
              {defaultDns.map((dns) => (
                <li key={dns}>
                  <p className="text-text-secondary text-sm whitespace-pre-line">
                    - {dns}
                  </p>
                </li>
              ))}
            </ul>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
