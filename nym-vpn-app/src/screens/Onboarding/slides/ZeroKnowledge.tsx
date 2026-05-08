import { useTranslation } from 'react-i18next';
import { ZeroKnowledge as ZeroKnowledgeAsset } from '../../../assets';

function ZeroKnowledge() {
  const { t } = useTranslation('onboarding');
  return (
    <div className="flex flex-col items-center gap-4">
      <ZeroKnowledgeAsset className="h-full max-h-64 w-fit" />
      <h1 className="text-2xl text-text-primary">
        {t('zero-knowledge.title')}
      </h1>
      <p className="text-center text-base text-text-secondary">
        {t('zero-knowledge.description')}
      </p>
    </div>
  );
}

export default ZeroKnowledge;
