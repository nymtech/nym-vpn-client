import { Favorite } from './tauri';

/**
 * The minimum a node needs to identify itself as a favorite. Kept structural
 * rather than aliasing the `Ui*` types so it can be used while those objects are
 * still being built.
 */
export type FavoritableNode =
  | { nodeType: 'country'; code: string }
  | { nodeType: 'region'; name: string }
  | { nodeType: 'gateway'; id: string };

/**
 * Stable string key for a favorite, used for membership tests and comparison.
 *
 * Identifiers are used verbatim: the backend matches favorites by exact
 * structural equality, so normalizing case here would make the frontend
 * disagree with what is stored.
 */
export function favoriteKey(favorite: Favorite): string {
  if ('country' in favorite) return `country:${favorite.country.code}`;
  if ('gateway' in favorite) return `gateway:${favorite.gateway.id}`;
  return `region:${favorite.region}`;
}

/** The favorite selector for a node row. */
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

/** Whether a node row is favorited, given the favorites of its hop. */
export function isNodeFavorite(
  node: FavoritableNode,
  favoriteKeys: ReadonlySet<string>,
): boolean {
  return favoriteKeys.has(favoriteKey(nodeToFavorite(node)));
}
