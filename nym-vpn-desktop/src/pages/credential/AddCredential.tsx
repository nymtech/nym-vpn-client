import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { NymIcon } from '../../assets';
import { Button, PageAnim, TextArea } from '../../ui';

function AddCredential() {
  const [phrase, setPhrase] = useState('');

  const { t } = useTranslation('addCredential');

  const onChange = (phrase: string) => {
    setPhrase(phrase);
  };

  const handleClick = async () => {};

  return (
    <PageAnim className="h-full flex flex-col justify-end items-center gap-10">
      <NymIcon className="w-32 h-32 fill-ghost dark:fill-baltic-sea-jaguar" />
      <div className="flex flex-col items-center gap-4 px-4">
        <h1 className="text-2xl dark:text-white">{t('welcome')}</h1>
        <h2 className="text-center dark:text-laughing-jack">
          {t('description1')}
        </h2>
        <p className="text-xs text-center text-dim-gray dark:text-mercury-mist w-4/5">
          {t('description2')}
        </p>
      </div>
      <TextArea
        value={phrase}
        onChange={onChange}
        spellCheck={false}
        resize="none"
        rows={10}
        label={t('input-label')}
      />
      <Button onClick={handleClick}>{t('add-button')}</Button>
    </PageAnim>
  );
}

export default AddCredential;
