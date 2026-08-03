import { useMemo } from 'react';
import { useShallow } from 'zustand/react/shallow';
import {
  Country,
  Gateway,
  GatewaysByCountry,
  NodeHop,
  Region,
  SelectedNode,
} from '../types';
import {
  GwSelectedKind,
  UiCountry,
  UiGateway,
  UiGatewaysByCountry,
  UiRegion,
  isSelectedNodeType,
} from '../types/node';
import { favoriteKey, isNodeFavorite } from '../types/favorites';
import { useAppStore } from '../store';
import { useFavorites } from '../store/favoritesState';
import useLang from './useLang';

function countryToUi(
  country: Country,
  selectedEntry: SelectedNode,
  selectedExit: SelectedNode,
  favoriteKeys: ReadonlySet<string>,
): UiCountry {
  return {
    ...country,
    nodeType: 'country',
    isSelected: isSelectedNodeType(country, selectedEntry, selectedExit),
    isFavorite: isNodeFavorite(
      { nodeType: 'country', code: country.code },
      favoriteKeys,
    ),
  };
}

function gatewaysToUi(
  gateways: Gateway[],
  selectedEntry: SelectedNode,
  selectedExit: SelectedNode,
  quicFilter: boolean,
  favoriteKeys: ReadonlySet<string>,
): UiGateway[] {
  return gateways.reduce<UiGateway[]>((acc, gw) => {
    if (quicFilter && !gw.quic) return acc;
    acc.push({
      ...gw,
      nodeType: 'gateway',
      isSelected: isSelectedNodeType(
        gw,
        selectedEntry,
        selectedExit,
      ) as GwSelectedKind,
      isFavorite: isNodeFavorite(
        { nodeType: 'gateway', id: gw.id },
        favoriteKeys,
      ),
    });
    return acc;
  }, []);
}

function regionsToUi(
  regions: Region[],
  selectedEntry: SelectedNode,
  selectedExit: SelectedNode,
  quicFilter: boolean,
  favoriteKeys: ReadonlySet<string>,
): UiRegion[] {
  return regions.reduce<UiRegion[]>((acc, region) => {
    const gateways = gatewaysToUi(
      region.gateways,
      selectedEntry,
      selectedExit,
      quicFilter,
      favoriteKeys,
    );
    if (gateways.length === 0) return acc;
    acc.push({
      ...region,
      nodeType: 'region',
      gateways,
      isSelected: isSelectedNodeType(region, selectedEntry, selectedExit),
      isFavorite: isNodeFavorite(
        { nodeType: 'region', name: region.name },
        favoriteKeys,
      ),
    });
    return acc;
  }, []);
}

function buildNodeList(
  list: GatewaysByCountry[],
  selectedEntry: SelectedNode,
  selectedExit: SelectedNode,
  quicFilter: boolean,
  getCountryName: (code: string) => string | null | undefined,
  compare: (a: string, b: string) => number,
  favoriteKeys: ReadonlySet<string>,
): UiGatewaysByCountry[] {
  return list
    .reduce<UiGatewaysByCountry[]>((acc, gwByCountry) => {
      if (quicFilter && !gwByCountry.quic) return acc;

      const gateways = gatewaysToUi(
        gwByCountry.gateways,
        selectedEntry,
        selectedExit,
        quicFilter,
        favoriteKeys,
      );
      if (gateways.length === 0) return acc;

      const country = countryToUi(
        gwByCountry.country,
        selectedEntry,
        selectedExit,
        favoriteKeys,
      );

      // Defensive check: regions structure changed in 1.18.0; cached data
      // from older versions may not have the array shape yet.
      const regions = Array.isArray(gwByCountry.regions)
        ? regionsToUi(
            gwByCountry.regions,
            selectedEntry,
            selectedExit,
            quicFilter,
            favoriteKeys,
          )
        : [];

      acc.push({
        country,
        regions,
        gateways,
        type: gwByCountry.type,
        isSelected: country.isSelected,
        i18n:
          getCountryName(gwByCountry.country.code) || gwByCountry.country.name,
      });
      return acc;
    }, [])
    .sort((a, b) => compare(a.i18n, b.i18n));
}

export function useNodeListData(hop: NodeHop) {
  const { compare, getCountryName } = useLang();
  const favorites = useFavorites(hop);
  const favoriteKeys = useMemo(
    () => new Set(favorites.map(favoriteKey)),
    [favorites],
  );

  const {
    vpnMode,
    entryNode,
    exitNode,
    quic,
    backendFlags,
    mxEntry,
    mxExit,
    wg,
    mxEntryLoading,
    mxExitLoading,
    wgLoading,
    mxEntryError,
    mxExitError,
    wgError,
  } = useAppStore(
    useShallow((s) => ({
      vpnMode: s.vpnMode,
      entryNode: s.entryNode,
      exitNode: s.exitNode,
      quic: s.quic,
      backendFlags: s.backendFlags,
      mxEntry: s.mxEntry,
      mxExit: s.mxExit,
      wg: s.wg,
      mxEntryLoading: s.mxEntryLoading,
      mxExitLoading: s.mxExitLoading,
      wgLoading: s.wgLoading,
      mxEntryError: s.mxEntryError,
      mxExitError: s.mxExitError,
      wgError: s.wgError,
    })),
  );

  const quicFilter =
    vpnMode === 'wg' && hop === 'entry' && backendFlags.quic && quic;

  const nodes = useMemo(() => {
    let rawList: GatewaysByCountry[];
    if (vpnMode === 'mixnet' && hop === 'entry') rawList = mxEntry;
    else if (vpnMode === 'mixnet' && hop === 'exit') rawList = mxExit;
    else rawList = wg;
    return buildNodeList(
      rawList,
      entryNode,
      exitNode,
      quicFilter,
      getCountryName,
      compare,
      favoriteKeys,
    );
  }, [
    vpnMode,
    hop,
    mxEntry,
    mxExit,
    wg,
    entryNode,
    exitNode,
    quicFilter,
    getCountryName,
    compare,
    favoriteKeys,
  ]);

  const gateways = useMemo(() => {
    const flat: UiGateway[] = [];
    for (const country of nodes) flat.push(...country.gateways);
    return flat.sort((a, b) => compare(a.name, b.name));
  }, [nodes, compare]);

  const loading = useMemo(() => {
    if (nodes.length > 0) return false;
    if (vpnMode === 'mixnet' && hop === 'entry') return mxEntryLoading;
    if (vpnMode === 'mixnet' && hop === 'exit') return mxExitLoading;
    return wgLoading;
  }, [nodes.length, vpnMode, hop, mxEntryLoading, mxExitLoading, wgLoading]);

  const error = useMemo(() => {
    if (vpnMode === 'mixnet' && hop === 'entry') return mxEntryError;
    if (vpnMode === 'mixnet' && hop === 'exit') return mxExitError;
    return wgError;
  }, [vpnMode, hop, mxEntryError, mxExitError, wgError]);

  return { nodes, gateways, loading, error, vpnMode, quicFilter };
}
