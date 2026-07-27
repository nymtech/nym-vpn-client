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
import { useAppStore } from '../store';
import useLang from './useLang';

// Membership set of the current hop's favorites, keyed as `${kind}:${value}`.
function favKey(kind: string, value: string) {
  return `${kind}:${value}`;
}

function countryToUi(
  country: Country,
  selectedEntry: SelectedNode,
  selectedExit: SelectedNode,
  favSet: Set<string>,
): UiCountry {
  return {
    ...country,
    nodeType: 'country',
    isSelected: isSelectedNodeType(country, selectedEntry, selectedExit),
    isFavorite: favSet.has(favKey('country', country.code.toUpperCase())),
  };
}

function gatewaysToUi(
  gateways: Gateway[],
  selectedEntry: SelectedNode,
  selectedExit: SelectedNode,
  quicFilter: boolean,
  favSet: Set<string>,
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
      isFavorite: favSet.has(favKey('gateway', gw.id)),
    });
    return acc;
  }, []);
}

function regionsToUi(
  regions: Region[],
  selectedEntry: SelectedNode,
  selectedExit: SelectedNode,
  quicFilter: boolean,
  favSet: Set<string>,
): UiRegion[] {
  return regions.reduce<UiRegion[]>((acc, region) => {
    const gateways = gatewaysToUi(
      region.gateways,
      selectedEntry,
      selectedExit,
      quicFilter,
      favSet,
    );
    if (gateways.length === 0) return acc;
    acc.push({
      ...region,
      nodeType: 'region',
      gateways,
      isSelected: isSelectedNodeType(region, selectedEntry, selectedExit),
    });
    return acc;
  }, []);
}

function buildNodeList(
  list: GatewaysByCountry[],
  selectedEntry: SelectedNode,
  selectedExit: SelectedNode,
  quicFilter: boolean,
  favSet: Set<string>,
  getCountryName: (code: string) => string | null | undefined,
  compare: (a: string, b: string) => number,
): UiGatewaysByCountry[] {
  return list
    .reduce<UiGatewaysByCountry[]>((acc, gwByCountry) => {
      if (quicFilter && !gwByCountry.quic) return acc;

      const gateways = gatewaysToUi(
        gwByCountry.gateways,
        selectedEntry,
        selectedExit,
        quicFilter,
        favSet,
      );
      if (gateways.length === 0) return acc;

      const country = countryToUi(
        gwByCountry.country,
        selectedEntry,
        selectedExit,
        favSet,
      );

      // Defensive check: regions structure changed in 1.18.0; cached data
      // from older versions may not have the array shape yet.
      const regions = Array.isArray(gwByCountry.regions)
        ? regionsToUi(
            gwByCountry.regions,
            selectedEntry,
            selectedExit,
            quicFilter,
            favSet,
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

  const {
    vpnMode,
    entryNode,
    exitNode,
    algo,
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
    favorites,
  } = useAppStore(
    useShallow((s) => ({
      vpnMode: s.vpnMode,
      entryNode: s.entryNode,
      exitNode: s.exitNode,
      algo: s.gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm,
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
      favorites: s.favorites,
    })),
  );

  const favSet = useMemo(
    () => new Set(favorites[hop].map((f) => favKey(f.kind, f.value))),
    [favorites, hop],
  );

  const effectiveEntry: SelectedNode =
    algo === 'explicit' ? entryNode : 'random';
  const effectiveExit: SelectedNode = algo === 'auto' ? 'random' : exitNode;

  const quicFilter =
    vpnMode === 'wg' && hop === 'entry' && backendFlags.quic && quic;

  const nodes = useMemo(() => {
    let rawList: GatewaysByCountry[];
    if (vpnMode === 'mixnet' && hop === 'entry') rawList = mxEntry;
    else if (vpnMode === 'mixnet' && hop === 'exit') rawList = mxExit;
    else rawList = wg;
    return buildNodeList(
      rawList,
      effectiveEntry,
      effectiveExit,
      quicFilter,
      favSet,
      getCountryName,
      compare,
    );
  }, [
    vpnMode,
    hop,
    mxEntry,
    mxExit,
    wg,
    effectiveEntry,
    effectiveExit,
    quicFilter,
    favSet,
    getCountryName,
    compare,
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
