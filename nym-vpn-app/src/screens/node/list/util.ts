import { Score } from '../../../types';
import { UiGateway, UiGatewaysByCountry, UiRegion } from '../../../types/node';

const scoreOrder: Record<Score, number> = {
  offline: 0,
  low: 1,
  medium: 2,
  high: 3,
};

export function sortByScore(a: Score, b: Score): number {
  if (a === b) {
    return 0;
  }
  return scoreOrder[b] - scoreOrder[a];
}

/**
 * Narrows a flat gateway list to those matching a search term, testing gateway
 * name, city, country name and id. Input order is preserved, so a recency-ordered
 * list stays recency-ordered.
 */
export function searchGateways(
  gateways: UiGateway[],
  search: string,
  getCountryName: (code: string) => string | null | undefined,
): UiGateway[] {
  const term = search.trim().toLowerCase();
  if (term.length === 0) return gateways;

  return gateways.filter((gw) => {
    const country = getCountryName(gw.country.code) || gw.country.name;
    return (
      gw.name.toLowerCase().includes(term) ||
      gw.location.city.toLowerCase().includes(term) ||
      country.toLowerCase().includes(term) ||
      gw.id.toLowerCase().includes(term)
    );
  });
}

function regionToFavorites(
  region: UiRegion,
  countryIsFavorite: boolean,
): UiRegion | null {
  // A favorited country carries its whole subtree, so nothing below it is
  // filtered.
  if (countryIsFavorite || region.isFavorite) return region;

  const gateways = region.gateways.filter((gw) => gw.isFavorite);
  return gateways.length > 0 ? { ...region, gateways } : null;
}

/**
 * Narrows the node tree to favorited entities, preserving structure and order.
 *
 * A country is kept when it is itself favorited, or when it contains a favorited
 * region or gateway. A favorited country contributes its full subtree; otherwise
 * only its favorited regions (with their full gateway sets) and its favorited
 * gateways are kept. Countries left with no gateways are dropped, mirroring how
 * `buildNodeList` discards empty countries.
 *
 * Favorited gateways stay nested under their country rather than being lifted
 * out, so nothing is rendered twice when a country and one of its gateways are
 * both favorited.
 */
export function filterToFavorites(
  nodes: UiGatewaysByCountry[],
): UiGatewaysByCountry[] {
  return nodes.reduce<UiGatewaysByCountry[]>((acc, node) => {
    if (node.country.isFavorite) {
      acc.push(node);
      return acc;
    }

    const regions = node.regions.reduce<UiRegion[]>((regionAcc, region) => {
      const filtered = regionToFavorites(region, false);
      if (filtered) regionAcc.push(filtered);
      return regionAcc;
    }, []);

    // The country's flat gateway list must cover everything reachable below it:
    // its own favorited gateways plus every gateway carried in by a favorited
    // region. `NodeItem` renders the flat list for non-US countries and the
    // regions for the US, while the header count reads the flat list for both —
    // so a favorited region has to contribute here or its nodes go uncounted.
    const keep = new Set<string>();
    for (const gw of node.gateways) if (gw.isFavorite) keep.add(gw.id);
    for (const region of regions)
      for (const gw of region.gateways) keep.add(gw.id);

    // Filtering the original array rather than concatenating preserves the
    // performance ordering the backend applied.
    const gateways = node.gateways.filter((gw) => keep.has(gw.id));
    if (gateways.length === 0) return acc;

    acc.push({ ...node, gateways, regions });
    return acc;
  }, []);
}
