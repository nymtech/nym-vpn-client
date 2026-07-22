import { invoke } from '@tauri-apps/api/core';
import { useEffect, useRef } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { AccountState, TAccountSummary } from '../types';
import { dispatch, useAppStore } from '../store';

const ACCOUNT_STATES_REFRESH_SUMMARY: ReadonlySet<AccountState> = new Set([
  'ready',
  'bandwidth-exceeded',
  'status-not-active',
  'no-subscription',
  'max-device-reached',
]);

export function useAccountSummaryOnAccountState() {
  const { accountState, accountSyncing, initialized } = useAppStore(
    useShallow((s) => ({
      accountState: s.accountState,
      accountSyncing: s.accountSyncing,
      initialized: s.initialized,
    })),
  );

  const prevRef = useRef<AccountState | null | undefined>(undefined);
  const prevSyncingRef = useRef<boolean>(false);

  useEffect(() => {
    const prev = prevRef.current;
    prevRef.current = accountState;

    const prevSyncing = prevSyncingRef.current;
    prevSyncingRef.current = accountSyncing;

    if (
      !initialized ||
      (prev === accountState && prevSyncing === accountSyncing)
    )
      return;
    if (!accountState || !ACCOUNT_STATES_REFRESH_SUMMARY.has(accountState))
      return;

    let cancelled = false;
    invoke<TAccountSummary>('get_account_summary')
      .then((summary) => {
        if (!cancelled && summary) {
          dispatch({ type: 'set-account-summary', summary });
        }
      })
      .catch((err: unknown) => {
        console.error(
          'Failed to get account summary on account state change',
          err,
        );
      });

    return () => {
      cancelled = true;
    };
  }, [accountState, accountSyncing, initialized]);
}
