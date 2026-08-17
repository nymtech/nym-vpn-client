import { useTranslation } from 'react-i18next';

function RecentsEmpty() {
  const { t } = useTranslation('node-location');

  return (
    <div className="space-y-4 px-6 py-4" data-testid="recents-empty">
      <p className="text-text-primary truncate">{t('recents.empty.title')}</p>
      <p className="text-text-secondary whitespace-pre-line">
        {t('recents.empty.description')}
      </p>
    </div>
  );
}

export default RecentsEmpty;
