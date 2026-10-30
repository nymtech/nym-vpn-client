import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router';
import { useAppStore } from '../../store';
import { kvGet, kvSet } from '../../kvStore';
import { routes } from '../../router';
import { ExpiryReminder, getExpiryReminder } from './expiryReminder';

// One key per (tier, expiry window): a deeper tier or a renewal yields a new one.
const keyOf = (r: ExpiryReminder) => `${r.tier}|${r.validUntil}`;

/**
 * Drives the upcoming-expiry renewal reminder as a home-screen dialog, escalating
 * by tier. Mirrors the iOS/macOS reminder.
 *
 * The dialog has no timeout: it stays on Home until the user acts on it (Renew /
 * Get a plan, or Not now). Acting or dismissing marks the (tier, expiry window) as
 * shown, persisted in the KV store; a reminder the user hasn't acted on comes back
 * on the next Home visit or launch. Because the dialog lives in the Home tree,
 * leaving Home takes it down automatically.
 */
function useExpiryReminder() {
  const accountSummary = useAppStore((s) => s.accountSummary);
  const navigate = useNavigate();
  // Kept after closing so the dialog keeps its text during the fade-out;
  // `open` alone drives visibility.
  const [reminder, setReminder] = useState<ExpiryReminder | null>(null);
  const [open, setOpen] = useState(false);
  // The last key the user acted on, covering the window before the KV write lands.
  const actedKeyRef = useRef<string | null>(null);

  useEffect(() => {
    const due = getExpiryReminder(accountSummary);
    if (due.tier === 'none') {
      // Renewed, logged out or otherwise no longer due.
      setOpen(false);
      return;
    }

    const key = keyOf(due);
    let cancelled = false;
    void kvGet<string>('expiry-reminder-shown').then((shownKey) => {
      if (cancelled || shownKey === key || actedKeyRef.current === key) {
        return;
      }
      setReminder(due);
      setOpen(true);
    });

    return () => {
      cancelled = true;
    };
  }, [accountSummary]);

  // Both acting on the reminder and dismissing it count as shown, so it won't
  // reappear for this (tier, window).
  const onClose = () => {
    if (!reminder) {
      return;
    }
    const key = keyOf(reminder);
    actedKeyRef.current = key;
    void kvSet('expiry-reminder-shown', key);
    setOpen(false);
  };

  const onAction = () => {
    onClose();
    // Both plan kinds renew/subscribe at the plan picker. `/account` itself has no
    // index screen (only `/account/select-a-plan`), so routing there would land on
    // a blank outlet.
    navigate(routes.selectPlan);
  };

  return { reminder, isOpen: open, onAction, onClose };
}

export default useExpiryReminder;
