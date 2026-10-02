import { useEffect, useRef } from 'react';
import { TFunction } from 'i18next';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { useAppStore } from '../../store';
import { useToast } from '../../hooks';
import { kvGet, kvSet } from '../../kvStore';
import { routes } from '../../router';
import { ExpiryReminder, getExpiryReminder } from './expiryReminder';

const REMINDER_TOAST_ID = 'expiry-reminder';

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

/**
 * Surfaces the upcoming-expiry renewal reminder as a home-screen toast, escalating
 * by tier (7d/3d = warning, 24h/expired = error) with a message set per
 * subscription vs free pass, stating the actual days/hours left. The action
 * routes to the choose-plan surface. Mirrors the iOS/macOS reminder.
 *
 * Like the Apple sticky snackbar, the toast has no timeout: it stays on Home until
 * the user acts on it (action button, close button or swipe). Only that marks the
 * (tier, expiry window) as shown, persisted in the KV store; a reminder the user
 * hasn't acted on comes back on the next Home visit or launch. A deeper tier, or a
 * renewal (new validUntil), yields a new key and a fresh toast.
 */
function useExpiryReminderToast() {
  const accountSummary = useAppStore((s) => s.accountSummary);
  const { add, close } = useToast();
  const { t } = useTranslation('notifications');
  const navigate = useNavigate();
  // The reminder currently on screen, and the last one the user acted on
  // (covers the window before the KV write lands).
  const displayedRef = useRef<{ key: string; title: string } | null>(null);
  const actedKeyRef = useRef<string | null>(null);
  const closeRef = useRef(close);
  closeRef.current = close;

  // Removes the toast without counting it as acted on. Base UI calls `onClose`
  // on every close and doesn't say why, so `displayedRef` must be cleared before
  // closing: `onClose` ignores the close when the ref is empty.
  const dismissRef = useRef(() => {
    if (displayedRef.current) {
      displayedRef.current = null;
      closeRef.current(REMINDER_TOAST_ID);
    }
  });

  // The toast provider sits above the router, so a toast outlives the screen
  // that added it. Taking the reminder down when Home unmounts keeps it off
  // other screens, including the login screen after a logout.
  useEffect(() => {
    const dismiss = dismissRef.current;
    return () => dismiss();
  }, []);

  useEffect(() => {
    const reminder = getExpiryReminder(accountSummary);
    if (reminder.tier === 'none') {
      // Renewed, logged out or otherwise no longer due.
      dismissRef.current();
      return;
    }

    const key = `${reminder.tier}|${reminder.validUntil}`;
    const title = reminderTitle(t, reminder);
    if (actedKeyRef.current === key) {
      return;
    }
    // Same reminder already on screen; only refresh it when the text changed
    // (e.g. a day has passed within the same tier).
    const displayed = displayedRef.current;
    if (displayed?.key === key && displayed.title === title) {
      return;
    }

    let cancelled = false;
    kvGet<string>('expiry-reminder-shown').then((shownKey) => {
      if (cancelled || shownKey === key) {
        return;
      }
      displayedRef.current = { key, title };

      const isError = reminder.tier === 'hour24' || reminder.tier === 'expired';
      // Re-adding under the same id updates the toast in place, onClose included.
      add({
        id: REMINDER_TOAST_ID,
        title,
        type: isError ? 'error' : 'warn',
        timeout: 0,
        onClose: () => {
          // Programmatic removal clears `displayedRef` beforehand; anything
          // else here is the user dismissing it or taking the action.
          if (displayedRef.current?.key !== key) {
            return;
          }
          displayedRef.current = null;
          actedKeyRef.current = key;
          kvSet('expiry-reminder-shown', key);
        },
        actionProps: {
          children: reminder.isFreepass
            ? t('renewal-reminder.action.get-plan')
            : t('renewal-reminder.action.renew'),
          onClick: () => {
            // Both plan kinds renew/subscribe at the plan picker. `/account` itself
            // has no index screen (only `/account/select-a-plan`), so routing there
            // would land on a blank outlet.
            close(REMINDER_TOAST_ID);
            navigate(routes.selectPlan);
          },
        },
      });
    });

    return () => {
      cancelled = true;
    };
  }, [accountSummary, add, close, t, navigate]);
}

export default useExpiryReminderToast;
