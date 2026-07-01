import { afterEach, describe, expect, it } from 'vitest';
import type { AppError, Country, Gateway, GatewaysByCountry } from '../types';
import { initialState } from '../store/slices/createMainSlice';
import { renderHookWithProviders, seedStore } from '../test/harness';
import { useNodeListData } from './useNodeListData';

const country = (code: string, name: string): Country => ({ code, name });

function gateway(
  id: string,
  code: string,
  name: string,
  quic: boolean,
): Gateway {
  return {
    id,
    name,
    country: country(code, name),
    quic,
  } as unknown as Gateway;
}

function gwByCountry(
  code: string,
  name: string,
  gateways: Gateway[],
  quic: boolean,
): GatewaysByCountry {
  return {
    country: country(code, name),
    regions: [],
    gateways,
    type: 'mixnet',
    quic,
  } as unknown as GatewaysByCountry;
}

afterEach(() => {
  // Reset the singleton store's derivation inputs to defaults.
  seedStore({
    ...initialState,
    mxEntry: [],
    mxExit: [],
    wg: [],
    mxEntryLoading: false,
    mxExitLoading: false,
    wgLoading: false,
    mxEntryError: null,
    mxExitError: null,
    wgError: null,
  });
});

describe('useNodeListData', () => {
  it('builds the mixnet entry list from mxEntry when mode=mixnet, hop=entry', () => {
    seedStore({
      vpnMode: 'mixnet',
      mxEntry: [
        gwByCountry(
          'DE',
          'Germany',
          [gateway('g1', 'DE', 'gw-de', false)],
          false,
        ),
      ],
      wg: [
        gwByCountry(
          'FR',
          'France',
          [gateway('g2', 'FR', 'gw-fr', false)],
          false,
        ),
      ],
    });

    const { result } = renderHookWithProviders(() => useNodeListData('entry'));

    expect(result.current.nodes).toHaveLength(1);
    expect(result.current.nodes[0].country.code).toBe('DE');
    expect(result.current.gateways.map((g) => g.id)).toEqual(['g1']);
    expect(result.current.vpnMode).toBe('mixnet');
  });

  it('reads mxExit for the exit hop in mixnet mode', () => {
    seedStore({
      vpnMode: 'mixnet',
      mxEntry: [
        gwByCountry('DE', 'Germany', [gateway('g1', 'DE', 'a', false)], false),
      ],
      mxExit: [
        gwByCountry('FR', 'France', [gateway('g2', 'FR', 'b', false)], false),
      ],
    });

    const { result } = renderHookWithProviders(() => useNodeListData('exit'));

    expect(result.current.gateways.map((g) => g.id)).toEqual(['g2']);
  });

  it('falls back to the wg list outside mixnet mode', () => {
    seedStore({
      vpnMode: 'wg',
      wg: [
        gwByCountry(
          'FR',
          'France',
          [gateway('g2', 'FR', 'gw-fr', false)],
          false,
        ),
      ],
    });

    const { result } = renderHookWithProviders(() => useNodeListData('entry'));

    expect(result.current.gateways.map((g) => g.id)).toEqual(['g2']);
  });

  it('sorts countries by their localized name', () => {
    seedStore({
      vpnMode: 'mixnet',
      mxEntry: [
        gwByCountry(
          'US',
          'United States',
          [gateway('g2', 'US', 'z', false)],
          false,
        ),
        gwByCountry('DE', 'Germany', [gateway('g1', 'DE', 'a', false)], false),
      ],
    });

    const { result } = renderHookWithProviders(() => useNodeListData('entry'));

    expect(result.current.nodes.map((n) => n.country.code)).toEqual([
      'DE',
      'US',
    ]);
  });

  it('applies the quic filter for wg entry when quic and the flag are enabled', () => {
    seedStore({
      vpnMode: 'wg',
      quic: true,
      backendFlags: {
        quic: true,
        domainFronting: false,
        zknymCredential: false,
      },
      wg: [
        gwByCountry(
          'DE',
          'Germany',
          [
            gateway('g1', 'DE', 'no-quic', false),
            gateway('g2', 'DE', 'quic', true),
          ],
          true,
        ),
      ],
    });

    const { result } = renderHookWithProviders(() => useNodeListData('entry'));

    expect(result.current.quicFilter).toBe(true);
    expect(result.current.gateways.map((g) => g.id)).toEqual(['g2']);
  });

  it('does not filter by quic when the backend flag is off', () => {
    seedStore({
      vpnMode: 'wg',
      quic: true,
      backendFlags: {
        quic: false,
        domainFronting: false,
        zknymCredential: false,
      },
      wg: [
        gwByCountry(
          'DE',
          'Germany',
          [
            gateway('g1', 'DE', 'no-quic', false),
            gateway('g2', 'DE', 'quic', true),
          ],
          false,
        ),
      ],
    });

    const { result } = renderHookWithProviders(() => useNodeListData('entry'));

    expect(result.current.quicFilter).toBe(false);
    expect(result.current.gateways).toHaveLength(2);
  });

  it('reports loading only while the relevant list is empty', () => {
    seedStore({ vpnMode: 'mixnet', mxEntry: [], mxEntryLoading: true });

    const { result } = renderHookWithProviders(() => useNodeListData('entry'));

    expect(result.current.loading).toBe(true);
  });

  it('is not loading once the relevant list has entries', () => {
    seedStore({
      vpnMode: 'mixnet',
      mxEntryLoading: true,
      mxEntry: [
        gwByCountry('DE', 'Germany', [gateway('g1', 'DE', 'a', false)], false),
      ],
    });

    const { result } = renderHookWithProviders(() => useNodeListData('entry'));

    expect(result.current.loading).toBe(false);
  });

  it('surfaces the error for the active mode and hop', () => {
    const err = { key: 'unknown', message: 'boom' } as unknown as AppError;
    seedStore({
      vpnMode: 'mixnet',
      mxExitError: err,
    });

    const { result } = renderHookWithProviders(() => useNodeListData('exit'));

    expect(result.current.error).toEqual(err);
  });
});
