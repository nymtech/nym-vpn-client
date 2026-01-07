import { useTranslation } from 'react-i18next';
import { ZeroKnowledge as ZeroKnowledgeAsset } from '../../../assets';

function ZeroKnowledge() {
  const { t } = useTranslation('onboarding');
  return (
    <div className="flex flex-col items-center gap-4">
      <ZeroKnowledgeAsset className="h-fit w-full max-w-72" />
      <h1 className="text-2xl text-baltic-sea dark:text-white">
        {t('zero-knowledge.title')}
      </h1>
      <p className="text-center text-base text-iron dark:text-bombay">
        {t('zero-knowledge.description')}
      </p>
    </div>
  );
}

export default ZeroKnowledge;
