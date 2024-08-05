import dayjs from 'dayjs';
import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { LngTag } from '../i18n';
import { kvSet } from '../kvStore';

function useLang() {
  const { i18n } = useTranslation();

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
