import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { useTranslation } from 'react-i18next';
import { useToast } from './index';

/* Access the system clipboard */
function useClipboard() {
  const { add } = useToast();
  const { t } = useTranslation('notifications');

  // Writes text to the clipboard
  const copy = async (text: string, notify = true) => {
    try {
      await writeText(text);
      if (notify) {
        add({
          title: t('copied-to-clipboard'),
          type: 'success',
        });
      }
    } catch (e) {
      console.error('failed to copy to clipboard', e);
    }
  };

  return { copy };
}

export default useClipboard;
