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
      console.log('set language:', lng);
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

  return { set };
}

export default useLang;
