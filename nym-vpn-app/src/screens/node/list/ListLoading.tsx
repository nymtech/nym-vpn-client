import { motion } from 'motion/react';
import { useTranslation } from 'react-i18next';

function ListLoading() {
  const { t } = useTranslation('node-location');

  return (
    <motion.div
      className="text-text-secondary mt-4 flex justify-center text-base"
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.2, ease: 'easeOut' }}
      data-testid="node-loading-indicator"
    >
      {t('loading')}
    </motion.div>
  );
}

export default ListLoading;
