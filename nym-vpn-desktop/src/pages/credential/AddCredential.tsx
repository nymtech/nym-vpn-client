import { invoke } from '@tauri-apps/api';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'framer-motion';
import { useNavigate } from 'react-router-dom';
import { NymDarkOutlineIcon, NymIcon } from '../../assets';
import { useMainState, useNotifications } from '../../contexts';
import { routes } from '../../router';
import { CmdError } from '../../types';
import { Button, PageAnim, TextArea } from '../../ui';

function AddCredential() {
  const { uiTheme } = useMainState();
  const [credential, setCredential] = useState('');
  const [error, setError] = useState<string | null>(null);

  const { push } = useNotifications();
  const navigate = useNavigate();
  const { t } = useTranslation('addCredential');

  const onChange = (credential: string) => {
    setCredential(credential);
    if (credential.length == 0) {
      setError(null);
    }
  };

  const handleClick = async () => {
    await invoke('add_credential', { credential: credential })
      .then(() => {
        navigate(routes.root);
        push({
          text: t('added-notification'),
          position: 'top',
          closeIcon: true,
        });
      })
      .catch((e: CmdError) => {
        console.warn('backend error:', e);
        setError(t('error'));
      });
  };

  return (
    <PageAnim className="h-full flex flex-col justify-end items-center gap-10 select-none cursor-default">
      {uiTheme === 'Dark' ? (
        <NymDarkOutlineIcon className="w-32 h-32" />
      ) : (
        <NymIcon className="w-32 h-32 fill-ghost" />
      )}
      <div className="flex flex-col items-center gap-4 px-4">
        <h1 className="text-2xl dark:text-white">{t('welcome')}</h1>
        <h2 className="text-center dark:text-laughing-jack">
          {t('description1')}
        </h2>
        <p className="text-xs text-center text-dim-gray dark:text-mercury-mist w-5/6">
          {t('description2')}
        </p>
      </div>
      <div className="w-full">
        <TextArea
          value={credential}
          onChange={onChange}
          spellCheck={false}
          resize="none"
          rows={10}
          label={t('input-label')}
        />
        {error ? (
          <motion.div
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.15, ease: 'easeInOut' }}
            className="text-teaberry h-3"
          >
            {error}
          </motion.div>
        ) : (
          <div className="h-3"></div>
        )}
      </div>
      <Button onClick={handleClick}>{t('add-button')}</Button>
    </PageAnim>
  );
}

export default AddCredential;
