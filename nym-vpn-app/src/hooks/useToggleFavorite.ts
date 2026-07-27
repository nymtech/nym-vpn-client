import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Favorite, NodeHop } from '../types';
import { useFavoritesStore } from '../store/favoritesState';
import useToast from './useToast';

function useToggleFavorite(hop: NodeHop) {
  const addFavorite = useFavoritesStore((s) => s.add);
  const removeFavorite = useFavoritesStore((s) => s.remove);
  const { add: addToast } = useToast();
  const { t } = useTranslation('node-location');

  return useCallback(
    async (favorite: Favorite, isFavorite: boolean) => {
      if (isFavorite) {
        removeFavorite(hop, favorite);
      } else {
        addFavorite(hop, favorite);
      }

      try {
        await invoke(isFavorite ? 'remove_favorite' : 'add_favorite', {
          hop,
          favorite,
        });
      } catch (error: unknown) {
        console.error('failed to update favorite', error);
        if (isFavorite) {
          addFavorite(hop, favorite);
        } else {
          removeFavorite(hop, favorite);
        }
        addToast({
          id: 'favorite-error',
          title: t('favorites.error'),
          type: 'error',
        });
      }
    },
    [hop, addFavorite, removeFavorite, addToast, t],
  );
}

export default useToggleFavorite;
