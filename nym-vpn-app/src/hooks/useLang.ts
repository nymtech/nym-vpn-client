import dayjs from 'dayjs';
import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { LngTag } from '../i18n';
import { kvSet } from '../kvStore';

/**
 * Hook to set the i18n language
 *
 * @returns The `set` function
 */
function useLang() {
  const { i18n } = useTranslation();

  /**
   * Sets the i18n language.
   * Also updates dayjs locale accordingly and saves
   * the language to the KV store
   *
   * @param lng - The language tag to set
   */
  const set = useCallback(
    async (lng: LngTag, updateDb = true) => {
      if (i18n.language === lng) {
        return;
      }
      console.info('set language:', lng);
      if (updateDb) {
        kvSet('UiLanguage', lng);
      }
      await i18n.changeLanguage(lng);
      switch (lng) {
        case 'zh-Hans':
          dayjs.locale('zh-cn');
          break;
        case 'pt-BR':
          dayjs.locale('pt-br');
          break;
        default:
          dayjs.locale(lng);
      }
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
      const regionNames = new Intl.DisplayNames(i18n.language, {
        type: 'region',
        fallback: 'none',
        style: 'long',
      });
      let name = null;
      try {
        name = regionNames.of(code);
      } catch (e) {
        console.warn(e);
      }
      return name;
    },
    [i18n.language],
  );

  /**
   * Compare two strings according to the sort order of the current language
   *
   * @param a - The first string to compare
   * @param b - The second string to compare
   */
  const compare = useCallback(
    (a: string, b: string) => {
      const collator = new Intl.Collator(i18n.language, {});
      return collator.compare(a, b);
    },
    [i18n.language],
  );

  return { compare, set, getCountryName };
}

export default useLang;
