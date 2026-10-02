import dayjs, { Dayjs } from 'dayjs';
import { TAccountSummary } from '../../types';

/**
 * Which tier of the upcoming-expiry renewal reminder applies, if any.
 *
 * A flat ladder shared across all plan kinds; only the message set — selected
 * by `isFreepass` — differs between subscription and free pass, not these
 * thresholds. Mirrors the Apple `ExpiryReminderTier` for cross-platform parity.
 */
export type ExpiryReminderTier =
  'none' | 'day7' | 'day3' | 'hour24' | 'expired';

export type ExpiryReminder = {
  tier: ExpiryReminderTier;
  isFreepass: boolean;
  // Unix-seconds expiry as a string, scoping dedupe/dismissal to one window.
  validUntil: string;
};

const HOURS_IN_24H = 24;
const HOURS_IN_3_DAYS = 72;
const HOURS_IN_7_DAYS = 168;

/**
 * Pure derivation of the reminder tier from an account summary. `now` is
 * injectable for testing. Gate order mirrors the Apple implementation:
 * no-subscription → pending → inactive (by effective state) → recurring/stacked
 * → ladder.
 */
export function getExpiryReminder(
  accountSummary?: TAccountSummary | null,
  now: Dayjs = dayjs(),
): ExpiryReminder {
  const subscription = accountSummary?.subscription?.subscription;
  const isFreepass = subscription?.kind === 'freepass';
  const validUntil = subscription ? String(subscription.validUntilUtc) : '0';
  const none: ExpiryReminder = { tier: 'none', isFreepass, validUntil };

  // No plan has ever existed → nothing to renew; never "expired".
  if (!accountSummary || !subscription) {
    return none;
  }
  // A pending purchase is surfaced elsewhere — never a renewal reminder.
  if (accountSummary.subscription?.status === 'pending') {
    return none;
  }

  // Treat as active when the flag says so OR the paid-until date is still in the
  // future — mirrors the apps' `isAccountActive()` fallback, so we never show
  // "expired" while the rest of the UI still treats the account as active.
  const validUntilDay = dayjs.unix(Number(subscription.validUntilUtc));
  const effectivelyActive =
    accountSummary.isSubscriptionActive || validUntilDay.isAfter(now);
  if (!effectivelyActive) {
    return { tier: 'expired', isFreepass, validUntil };
  }

  // Auto-renewing or stacked plans renew themselves — no reminder needed.
  if (subscription.isRecurring || accountSummary.isSubscriptionStacked) {
    return none;
  }

  // The account is active by definition here — the `!effectivelyActive` branch above
  // owns "expired" — so the ladder never returns "expired": a grace-period account
  // (active flag, date just past) gets the most urgent tier, not a false "expired".
  // Minutes avoid dayjs hour-truncation showing "expired" up to 59 min before expiry.
  const minutesRemaining = validUntilDay.diff(now, 'minute');
  if (minutesRemaining <= HOURS_IN_24H * 60) {
    return { tier: 'hour24', isFreepass, validUntil };
  }
  if (minutesRemaining <= HOURS_IN_3_DAYS * 60) {
    return { tier: 'day3', isFreepass, validUntil };
  }
  if (minutesRemaining <= HOURS_IN_7_DAYS * 60) {
    return { tier: 'day7', isFreepass, validUntil };
  }
  return none;
}
