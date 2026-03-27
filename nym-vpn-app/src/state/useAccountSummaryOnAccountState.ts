import { invoke } from '@tauri-apps/api/core';
import { useEffect, useRef } from 'react';
import { AccountState, StateDispatch, TAccountSummary } from '../types';

const ACCOUNT_STATES_REFRESH_SUMMARY: ReadonlySet<AccountState> = new Set([
  'ready',
  'upgrade-mode',
  'bandwidth-exceeded',
  'status-not-active',
  'no-subscription',
  'max-device-reached',
  'requesting-zk-nyms',
]);

export function useAccountSummaryOnAccountState(
  accountState: AccountState | null | undefined,
  accountSyncing: boolean,
  initialized: boolean,
  dispatch: StateDispatch,
) {
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
  }, [accountState, accountSyncing, initialized, dispatch]);
}
