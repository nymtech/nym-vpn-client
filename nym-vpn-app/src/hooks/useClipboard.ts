import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useTranslation } from 'react-i18next';
import { useInAppNotify } from '../contexts';

/* Access the system clipboard */
function useClipboard() {
  const { push } = useInAppNotify();
  const { t } = useTranslation('notifications');

  // Writes text to the clipboard
  const copy = async (text: string, notify = true) => {
    try {
      await writeText(text);
      if (notify) {
        push({
          message: t('copied-to-clipboard'),
          clickAway: true,
        });
      }
    } catch (e) {
      console.error('failed to copy to clipboard', e);
    }
  };

  return { copy };
}

export default useClipboard;
