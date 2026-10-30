import { useEffect, useRef, useState } from 'react';
import { TFunction } from 'i18next';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { useAppStore } from '../../store';
import { kvGet, kvSet } from '../../kvStore';
import { routes } from '../../router';
import { ExpiryReminder, getExpiryReminder } from './expiryReminder';

export type ExpiryReminderDialogProps = {
  isOpen: boolean;
  title: string;
  body: string;
  actionLabel: string;
  dismissLabel: string;
  onAction: () => void;
  onClose: () => void;
};

function reminderTitle(
  t: TFunction<'notifications', undefined>,
  { tier, isFreepass, count }: ExpiryReminder,
): string {
  const plan = isFreepass ? 'freepass' : 'subscription';
  switch (tier) {
    case 'day7':
    case 'day3':
      return t(`renewal-reminder.days.${plan}`, { count });
    case 'hour24':
      return t(`renewal-reminder.hours.${plan}`, { count });
    default:
      return t(`renewal-reminder.expired.${plan}`);
  }
}

function reminderBody(
  t: TFunction<'notifications', undefined>,
  { isFreepass }: ExpiryReminder,
): string {
  return isFreepass
    ? t('renewal-reminder.body.freepass')
    : t('renewal-reminder.body.subscription');
}

/**
 * Drives the upcoming-expiry renewal reminder as a home-screen dialog, escalating
 * by tier, with a message set per subscription vs free pass stating the actual
 * days/hours left. The action routes to the choose-plan surface. Mirrors the
 * iOS/macOS reminder.
 *
 * The dialog has no timeout: it stays on Home until the user acts on it (Renew /
 * Get a plan, or Not now). Acting or dismissing marks the (tier, expiry window) as
 * shown, persisted in the KV store; a reminder the user hasn't acted on comes back
 * on the next Home visit or launch. A deeper tier, or a renewal (new validUntil),
 * yields a new key and a fresh dialog. Because the dialog lives in the Home tree,
 * leaving Home takes it down automatically — no cross-screen cleanup is needed.
 */
function useExpiryReminder(): ExpiryReminderDialogProps {
  const accountSummary = useAppStore((s) => s.accountSummary);
  const { t } = useTranslation('notifications');
  const navigate = useNavigate();
  // The reminder currently on screen and its key.
  const [current, setCurrent] = useState<{
    reminder: ExpiryReminder;
    key: string;
  } | null>(null);
  // Mirror of what's on screen (key + remaining count), so a re-render doesn't
  // re-trigger work; and the last key the user acted on, covering the window
  // before the KV write lands.
  const displayedRef = useRef<{ key: string; count: number } | null>(null);
  const actedKeyRef = useRef<string | null>(null);

  useEffect(() => {
    const reminder = getExpiryReminder(accountSummary);
    if (reminder.tier === 'none') {
      // Renewed, logged out or otherwise no longer due.
      displayedRef.current = null;
      setCurrent(null);
      return;
    }

    const key = `${reminder.tier}|${reminder.validUntil}`;
    if (actedKeyRef.current === key) {
      return;
    }
    // Same reminder already on screen; only refresh when the remaining count
    // changed (e.g. a day passed within the same tier).
    const displayed = displayedRef.current;
    if (displayed?.key === key && displayed.count === reminder.count) {
      return;
    }

    let cancelled = false;
    void kvGet<string>('expiry-reminder-shown').then((shownKey) => {
      if (cancelled || shownKey === key) {
        return;
      }
      displayedRef.current = { key, count: reminder.count };
      setCurrent({ reminder, key });
    });

    return () => {
      cancelled = true;
    };
  }, [accountSummary]);

  // Both acting on the reminder and dismissing it count as shown, so it won't
  // reappear for this (tier, window).
  const markShown = (key: string) => {
    displayedRef.current = null;
    actedKeyRef.current = key;
    void kvSet('expiry-reminder-shown', key);
    setCurrent(null);
  };

  const onClose = () => {
    if (current) {
      markShown(current.key);
    }
  };

  const onAction = () => {
    if (current) {
      markShown(current.key);
    }
    // Both plan kinds renew/subscribe at the plan picker. `/account` itself has no
    // index screen (only `/account/select-a-plan`), so routing there would land on
    // a blank outlet.
    navigate(routes.selectPlan);
  };

  const reminder = current?.reminder ?? null;
  return {
    isOpen: reminder !== null,
    title: reminder ? reminderTitle(t, reminder) : '',
    body: reminder ? reminderBody(t, reminder) : '',
    actionLabel: reminder
      ? reminder.isFreepass
        ? t('renewal-reminder.action.get-plan')
        : t('renewal-reminder.action.renew')
      : '',
    dismissLabel: t('renewal-reminder.action.not-now'),
    onAction,
    onClose,
  };
}

export default useExpiryReminder;
