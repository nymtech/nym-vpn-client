import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { useAppStore } from '../../store';
import { useToast } from '../../hooks';
import { routes } from '../../router';
import { getExpiryReminder } from './expiryReminder';

const SHOWN_STORAGE_KEY = 'nym.expiryReminder.shown';
const REMINDER_TOAST_ID = 'expiry-reminder';
const REMINDER_TIMEOUT_MS = 10000;

function alreadyShown(key: string): boolean {
  try {
    return localStorage.getItem(SHOWN_STORAGE_KEY) === key;
  } catch {
    return false;
  }
}

function markShown(key: string) {
  try {
    localStorage.setItem(SHOWN_STORAGE_KEY, key);
  } catch {
    // localStorage unavailable (private mode / blocked): the in-memory ref still
    // dedupes within the session; the reminder may simply re-show next launch.
  }
}

/**
 * Surfaces the upcoming-expiry renewal reminder as a home-screen toast, escalating
 * by tier (7d/3d = warning, 24h/expired = error) with a message set per
 * subscription vs free pass. The action routes to the renew / choose-plan surface.
 * Mirrors the iOS/macOS reminder.
 *
 * A toast is transient, so — unlike the Apple sticky snackbar — this shows once per
 * (tier, expiry window): the key is persisted so it is not re-shown on every store
 * update or on the next launch for the same tier. A deeper tier, or a renewal (new
 * validUntil), yields a new key and a fresh toast.
 */
function useExpiryReminderToast() {
  const accountSummary = useAppStore((s) => s.accountSummary);
  const { add } = useToast();
  const { t } = useTranslation('notifications');
  const navigate = useNavigate();
  const shownKeyRef = useRef<string | null>(null);

  useEffect(() => {
    const { tier, isFreepass, validUntil } = getExpiryReminder(accountSummary);
    if (tier === 'none') {
      return;
    }

    const key = `${tier}|${validUntil}`;
    if (shownKeyRef.current === key || alreadyShown(key)) {
      return;
    }
    shownKeyRef.current = key;
    markShown(key);

    const isError = tier === 'hour24' || tier === 'expired';
    const planSuffix = isFreepass ? 'freepass' : 'subscription';

    add({
      id: REMINDER_TOAST_ID,
      title: t(`renewal-reminder.${tier}.${planSuffix}`),
      type: isError ? 'error' : 'warn',
      timeout: REMINDER_TIMEOUT_MS,
      actionProps: {
        children: isFreepass
          ? t('renewal-reminder.action.get-plan')
          : t('renewal-reminder.action.renew'),
        onClick: () => {
          navigate(isFreepass ? routes.selectPlan : routes.account);
        },
      },
    });
  }, [accountSummary, add, t, navigate]);
}

export default useExpiryReminderToast;
