import { useEffect, useState } from 'react';
import { useLocation } from 'react-router';
import { useTranslation } from 'react-i18next';
import { PageAnim } from '../../../../ui';
import { CodeDependency } from '../../../../types';

function LicenseDetails() {
  const [license, setLicense] = useState<CodeDependency | null>(null);
  const [language, setLanguage] = useState<'rust' | 'js' | null>(null);

  const { t } = useTranslation('licenses');
  const locationState = useLocation().state as {
    license: CodeDependency;
    language: string;
  };

  useEffect(() => {
    if (locationState.license) {
      setLicense(locationState.license);
    }
    if (locationState.language) {
      setLanguage(locationState.language as 'rust' | 'js');
    }
  }, [locationState]);

  const { licenses, name, repository, authors, version } = license || {};

  const label = (label: string) => (
    <p className="truncate text-iron dark:text-bombay select-none cursor-default">
      {label}:
    </p>
  );

  return (
    <PageAnim className="h-full flex flex-col">
      {license ? (
        <article className="flex flex-col gap-4">
          <div className="flex flex-row items-center gap-4">
            {label(t('name'))}
            <p className="truncate font-medium">{name}</p>
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
          <div className="flex flex-row items-center gap-4">
            {label(t('language'))}
            <p className="italic truncate">
              {language === 'js' ? 'JavaScript' : 'Rust'}
            </p>
          </div>
        </article>
      ) : (
        <span className="mt-4 pl-4 italic text-iron dark:text-bombay select-none cursor-default">
          {t('no-data')}
        </span>
      )}
    </PageAnim>
  );
}

export default LicenseDetails;
