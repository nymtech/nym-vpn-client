import { useEffect, useState } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import { motion } from 'motion/react';
import { Welcome as WelcomeAsset } from '../../../assets';

function Welcome() {
  const { t } = useTranslation('onboarding');
  const [showAsset, setShowAsset] = useState(false);

  useEffect(() => {
    const id = setTimeout(() => setShowAsset(true), 200);
    return () => clearTimeout(id);
  }, []);

  return (
    <div className="flex flex-col items-center gap-4">
      <div className="aspect-390/412 h-64 w-auto max-w-full shrink-0">
        {showAsset && (
          <motion.div
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.5, ease: [0.22, 1, 0.36, 1] }}
            className="h-full w-full"
          >
            <WelcomeAsset className="h-full w-full" />
          </motion.div>
        )}
      </div>
      <h1 className="text-text-primary text-2xl">{t('welcome.title')}</h1>
      <p className="text-text-secondary text-center text-sm whitespace-pre-line">
        <Trans
          i18nKey="welcome.description"
          ns="onboarding"
          components={{ large: <span className="text-base!" /> }}
        />
      </p>
    </div>
  );
}

export default Welcome;
