import { useEffect, useState } from 'react';
import { useLocation } from 'react-router-dom';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { PageAnim } from '../../../../ui';
import { CodeDependency } from '../../../../types';

function LicenseDetails() {
  const [license, setLicense] = useState<CodeDependency | null>(null);
  const [language, setLanguage] = useState<'rust' | 'js' | null>(null);

  const { t } = useTranslation('licenses');
  const { state } = useLocation();

  useEffect(() => {
    if (state.license) {
      setLicense(state.license as CodeDependency);
    }
    if (state.language) {
      setLanguage(state.language as 'rust' | 'js');
    }
  }, [state]);

  const { licenses, name, repository, authors, licenseTexts, version } =
    license || {};

  const label = (label: string) => (
    <p className="truncate text-dim-gray dark:text-mercury-mist select-none cursor-default">
      {label}:
    </p>
  );

  return (
    <PageAnim className="h-full flex flex-col">
      {license ? (
        <article className="flex flex-col gap-4">
          <div className="flex flex-row items-center gap-4">
            {label(t('name'))}
            <p className="truncate font-semibold">{name}</p>
          </div>
          <div className="flex flex-row items-center gap-4">
            {label(t('version'))}
            <p className="truncate">{version}</p>
          </div>
          <div className="flex flex-col gap-2">
            {label(t('licenses'))}
            {licenses && (
              <ul>
                {licenses.map((license) => (
                  <li className="truncate" key={license}>
                    {license}
                  </li>
                ))}
              </ul>
            )}
          </div>
          <div className="flex flex-col gap-2">
            {label(t('repository'))}
            {repository && (
              <a
                className="truncate hover:underline"
                href={repository}
                target="_blank"
                rel="noreferrer"
              >
                {repository}
              </a>
            )}
          </div>

          <div className="flex flex-col gap-2">
            {label(t('authors'))}
            {authors && (
              <ul>
                {authors.map((author) => (
                  <li className="truncate" key={author}>
                    {author}
                  </li>
                ))}
              </ul>
            )}
          </div>
          <div className="flex flex-col gap-2">
            {label(t('license-texts'))}
            {licenseTexts && (
              <ul className="flex flex-col gap-4">
                {licenseTexts.map(
                  (text, i) =>
                    text.length > 0 && (
                      <li
                        key={i}
                        className={clsx([
                          'text-sm break-words mr-4 overflow-scroll max-h-44 min-w-52',
                        ])}
                      >
                        {text}
                      </li>
                    ),
                )}
              </ul>
            )}
          </div>
          <div className="flex flex-row items-center gap-4">
            {label(t('language'))}
            <p className="italic truncate">
              {language === 'js' ? 'JavaScript' : 'Rust'}
            </p>
          </div>
        </article>
      ) : (
        <span className="mt-4 pl-4 italic text-dim-gray dark:text-mercury-mist select-none cursor-default">
          {t('no-data')}
        </span>
      )}
    </PageAnim>
  );
}

export default LicenseDetails;
