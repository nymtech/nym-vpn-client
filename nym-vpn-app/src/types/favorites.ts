import { Favorite } from './tauri';

export type FavoritableNode =
  | { nodeType: 'country'; code: string }
  | { nodeType: 'region'; name: string }
  | { nodeType: 'gateway'; id: string };

export function favoriteKey(favorite: Favorite): string {
  if ('country' in favorite) return `country:${favorite.country.code}`;
  if ('gateway' in favorite) return `gateway:${favorite.gateway.id}`;
  return `region:${favorite.region}`;
}

export function nodeToFavorite(node: FavoritableNode): Favorite {
  switch (node.nodeType) {
    case 'country':
      return { country: { code: node.code } };
    case 'region':
      return { region: node.name };
    case 'gateway':
      return { gateway: { id: node.id } };
  }
}

export function isNodeFavorite(
  node: FavoritableNode,
  favoriteKeys: ReadonlySet<string>,
): boolean {
  return favoriteKeys.has(favoriteKey(nodeToFavorite(node)));
}
