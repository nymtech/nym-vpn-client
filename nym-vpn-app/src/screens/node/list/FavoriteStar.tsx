import { Button } from '@headlessui/react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { MsIcon } from '../../../ui';

type FavoriteStarProps = {
  isFavorite: boolean;
  onToggle: () => void;
  className?: string;
};

// Star toggle used on location and gateway rows to (un)favorite them. Stops
// click propagation so toggling never triggers row selection/expansion.
function FavoriteStar({ isFavorite, onToggle, className }: FavoriteStarProps) {
  const { t } = useTranslation('node-location');

  return (
    <Button
      aria-label={t(isFavorite ? 'favorites.remove' : 'favorites.add')}
      aria-pressed={isFavorite}
      data-testid="favorite-star"
      data-test-favorite={isFavorite ? 'true' : 'false'}
      className={clsx(
        'flex shrink-0 cursor-default items-center justify-center rounded-full p-1 select-none',
        'focus:outline-none',
        isFavorite
          ? 'text-brand-primary'
          : 'text-text-secondary hover:text-brand-primary',
        className,
      )}
      onClick={(e) => {
        e.stopPropagation();
        onToggle();
      }}
    >
      <MsIcon icon="star" filled={isFavorite} className="text-xl!" />
    </Button>
  );
}

export default FavoriteStar;
