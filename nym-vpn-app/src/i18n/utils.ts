import { loadLocale, supportedLngs } from './config';
import { LngTag } from './types';

/**
 * Matches an OS locale string (e.g. "en-US", "zh-Hans-CN") to the closest
 * supported language tag. Falls back to 'en' if no match is found.
 */
export function matchSupportedLocale(osLocale: string): LngTag {
  // Try exact match first (e.g. "en" → "en")
  const lower = osLocale.toLowerCase();
  if ((supportedLngs as readonly string[]).includes(lower)) {
    return lower as LngTag;
  }

  // Try matching just the primary language subtag (e.g. "en-US" → "en", "zh-Hans-CN" → "zh")
  const lang = osLocale.split(/[-_]/)[0].toLowerCase();
  if ((supportedLngs as readonly string[]).includes(lang)) {
    return lang as LngTag;
  }

  return 'en';
}

/**
 * Detects the system locale and returns the best matching supported language tag.
 * Uses navigator.language as the locale source.
 * Falls back to 'en' if no translations are available for the detected locale.
 */
export async function detectSystemLocale(): Promise<LngTag> {
  const matched = matchSupportedLocale(navigator.language);
  if (matched === 'en') return 'en';

  // Verify that translation files exist for this locale before returning it.
  // Falls back to 'en' if the locale has no bundled translations.
  try {
    const resources = await loadLocale(matched);
    if (Object.keys(resources).length > 0) {
      return matched;
    }
  } catch {
    // translation files missing
  }

  return 'en';
}
