import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { dispatch, useAppStore } from '../../../../store';
import { useToast } from '../../../../hooks/index';

export const useGeoExclusion = () => {
  const { t } = useTranslation('settings');
  const { enabled, listenPort, excludedCountries } = useAppStore(
    (s) => s.geoExclusion,
  );
  const { add: addToast } = useToast();

  const setEnabled = async (next: boolean) => {
    try {
      await invoke('set_geo_exclusion_enabled', { enabled: next });
      dispatch({ type: 'set-geo-exclusion-enabled', enabled: next });
    } catch (error) {
      console.error('Failed to set geo exclusion enabled', error);
      addToast({
        id: 'geo-exclusion-enabled-error',
        title: t('geo-exclusion.errors.failed-to-start'),
        type: 'error',
      });
    }
  };

  const setPort = async (port: number) => {
    try {
      await invoke('set_geo_exclusion_listen_port', { port });
      dispatch({ type: 'set-geo-exclusion-listen-port', port });
    } catch (error) {
      console.error('Failed to set geo exclusion listen port', error);
      addToast({
        title: t('geo-exclusion.errors.failed-to-set-port'),
        type: 'error',
      });
    }
  };

  const setExcludedCountry = async (country: string) => {
    try {
      const countries = [country];
      await invoke('set_geo_exclusion_excluded_countries', { countries });
      dispatch({ type: 'set-geo-exclusion-excluded-countries', countries });
      return true;
    } catch (error) {
      console.error('Failed to set geo exclusion excluded countries', error);
      addToast({
        title: t('geo-exclusion.errors.failed-to-set-region'),
        type: 'error',
      });
      return false;
    }
  };

  return {
    enabled,
    listenPort,
    excludedCountries,
    setEnabled,
    setPort,
    setExcludedCountry,
  };
};
