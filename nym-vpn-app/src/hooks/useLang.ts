import dayjs from 'dayjs';
import { useCallback, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { LngTag, detectSystemLocale, loadLocale } from '../i18n';
import { kvDel, kvSet } from '../kvStore';

/**
 * Hook to set the i18n language
 *
 * @returns The `set` function
 */
function useLang() {
  const { i18n } = useTranslation();

  const regionNames = useMemo(() => {
    return new Intl.DisplayNames(i18n.language, {
      type: 'region',
      fallback: 'none',
      style: 'long',
    });
  }, [i18n.language]);

  const collator = useMemo(() => {
    return new Intl.Collator(i18n.language, {});
  }, [i18n.language]);

  /**
   * Sets the i18n language.
   * Also updates dayjs locale accordingly and saves
   * the language to the KV store
   *
   * @param lng - The language tag to set
   */
  const set = useCallback(
    async (lng: LngTag, updateDb = true) => {
      const locale = await loadLocale(lng);
      Object.entries(locale).forEach(([namespace, value]) => {
        i18n.addResourceBundle(lng, namespace, value, true, true);
      });

      console.info('set language:', lng);
      if (updateDb) {
        kvSet('ui-language', lng);
      }
      await i18n.changeLanguage(lng);
      dayjs.locale(lng);

      document.documentElement.setAttribute('dir', i18n.dir());
      document.documentElement.setAttribute('lang', lng);
    },
    [i18n],
  );

  /**
   * Get the localized country name
   *
   * @param code - Two-letter country code
   */
  const getCountryName = useCallback(
    (code: string) => {
      let name = null;
      try {
        name = regionNames.of(code);
      } catch (e) {
        console.warn(e);
      }
      return name;
    },
    [regionNames],
  );

  /**
   * Compare two strings according to the sort order of the current language
   *
   * @param a - The first string to compare
   * @param b - The second string to compare
   */
  const compare = useCallback(
    (a: string, b: string) => {
      return collator.compare(a, b);
    },
    [collator],
  );

  /**
   * Clears any stored language preference and applies the OS system language.
   */
  const setSystem = useCallback(async () => {
    await kvDel('ui-language');
    const lng = await detectSystemLocale();
    await set(lng, false);
  }, [set]);

  return { compare, set, setSystem, getCountryName };
}

export default useLang;
