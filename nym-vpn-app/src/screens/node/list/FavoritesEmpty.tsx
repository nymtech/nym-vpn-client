import { useTranslation } from 'react-i18next';

function FavoritesEmpty({ hasFavorites }: { hasFavorites: boolean }) {
  const { t } = useTranslation('node-location');

  return (
    <div className="space-y-4 px-6 py-4" data-testid="favorites-empty">
      <p className="text-text-primary truncate">
        {t(
          hasFavorites ? 'favorites.empty.unavailable' : 'favorites.empty.none',
        )}
      </p>
      <p className="text-text-secondary whitespace-pre-line">
        {t(
          hasFavorites
            ? 'favorites.empty.unavailable-description'
            : 'favorites.empty.none-description',
        )}
      </p>
    </div>
  );
}

export default FavoritesEmpty;
