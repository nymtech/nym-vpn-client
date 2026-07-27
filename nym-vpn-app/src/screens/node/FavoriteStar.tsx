import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { Button } from '@headlessui/react';
import { MsIcon } from '../../ui';
import { Favorite, NodeHop } from '../../types';
import { useToggleFavorite } from '../../hooks';

type Props = {
  favorite: Favorite;
  isFavorite: boolean;
  hop: NodeHop;
  className?: string;
};

function FavoriteStar({ favorite, isFavorite, hop, className }: Props) {
  const toggleFavorite = useToggleFavorite(hop);
  const { t } = useTranslation('node-location');

  return (
    <Button
      onClick={() => toggleFavorite(favorite, isFavorite)}
      aria-pressed={isFavorite}
      aria-label={t(isFavorite ? 'favorites.remove' : 'favorites.add')}
      title={t(isFavorite ? 'favorites.remove' : 'favorites.add')}
      data-testid="favorite-star"
      data-favorite={isFavorite}
      className={clsx(
        'flex shrink-0 cursor-default items-center justify-center rounded-full p-1 focus:outline-none',
        className,
      )}
    >
      <MsIcon
        icon="star"
        filled={isFavorite}
        className={clsx(
          'text-xl! transition-colors',
          isFavorite
            ? 'text-status-warning hover:text-status-warning/80'
            : 'text-text-secondary hover:text-text-primary',
        )}
      />
    </Button>
  );
}

export default FavoriteStar;
