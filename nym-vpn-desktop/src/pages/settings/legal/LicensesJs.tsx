import { useEffect, useState } from 'react';
import licensesUrl from '/licenses-js.json?url';

type LicensesJson = {
  [key: string]: {
    licenses?: string | string[];
    repository?: string;
    publisher?: string;
    email?: string;
    licenseText?: string;
    copyright?: string;
  };
};

type LicenseInfo = LicensesJson[keyof LicensesJson];

function LicensesJs() {
  const [licenses, setLicenses] = useState<LicensesJson>();
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch(licensesUrl)
      .then((response) => response.json())
      .then((data: LicensesJson) => {
        setLicenses(data);
        console.log('UP');
      })
      .catch((e) => {
        console.warn('Failed to fetch licenses data', e);
      });
  }, []);

  return (
    <div className="h-full flex flex-col mt-2 gap-6">
      {licenses &&
        Object.entries(licenses).map(([name, info]) => {
          return (
            <article key={name} className="flex flex-col gap-2 mb-3">
              <h2 className="text-lg font-bold">{name}</h2>
              {info.licenses && (
                <div>
                  {Array.isArray(info.licenses)
                    ? info.licenses.join(', ')
                    : info.licenses}
                </div>
              )}
              {info.repository && (
                <a
                  className="text-sm break-words hover:underline"
                  href={info.repository}
                  target="_blank"
                  rel="noreferrer"
                >
                  {info.repository}
                </a>
              )}
              {info.publisher && <span>{info.publisher}</span>}
              {info.email && (
                <a
                  className="text-sm break-words hover:underline"
                  href={`mailto:${info.email}`}
                >
                  {info.email}
                </a>
              )}
              {info.licenseText && (
                <div className="text-sm break-words">{info.licenseText}</div>
              )}
              {info.copyright && (
                <span className="text-sm break-words">{info.copyright}</span>
              )}
            </article>
          );
        })}
    </div>
  );
}

export default LicensesJs;
