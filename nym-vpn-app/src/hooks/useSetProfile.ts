import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { useFetchGateways, useFetchRecents } from '../store';
import { Profile, VpnMode } from '../types';
import useToast from './useToast';

const profileVpnMode: Record<Profile, VpnMode> = {
  safest: 'wg',
  mostPrivate: 'mixnet',
  fastest: 'wg',
  random: 'wg',
};

function useSetProfile() {
  const { t } = useTranslation();
  const { add } = useToast();
  const fetchGateways = useFetchGateways();
  const fetchRecents = useFetchRecents();

  return useCallback(
    async (profile: Profile): Promise<boolean> => {
      try {
        await invoke('set_profile', { profile });
      } catch (error: unknown) {
        console.error(`failed to set profile [${profile}]`, error);
        add({
          id: 'set-profile-error',
          title: t('profiles.error'),
          type: 'error',
        });
        return false;
      }
      // entry/exit/vpnMode/frontingMode land in the store via the daemon's
      // ConfigChanged event; the gateway lists must be refetched manually
      const mode = profileVpnMode[profile];
      if (mode === 'mixnet') {
        fetchGateways('mx-entry');
        fetchGateways('mx-exit');
      } else {
        fetchGateways('wg');
      }
      fetchRecents(mode);
      return true;
    },
    [t, add, fetchGateways, fetchRecents],
  );
}

export default useSetProfile;
