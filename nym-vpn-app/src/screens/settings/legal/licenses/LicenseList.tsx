import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { type } from '@tauri-apps/plugin-os';
import { List, type RowComponentProps } from 'react-window';
import { useMainState } from '../../../../contexts';
import { routes } from '../../../../router';
import { PageAnim, SettingsMenuCard } from '../../../../ui';
import { CodeDependency } from '../../../../types';

const os = type();
const heightFactorLinux = 7;
const heightFactor = 6;

const Row = ({
  index,
  style,
  licenses,
  language,
}: RowComponentProps<{
  licenses: CodeDependency[];
  language: 'js' | 'rust';
}>) => {
  const navigate = useNavigate();
  const license = licenses[index];
  const { name, version, licenses: pkgLicenses } = license;
  const description = Array.isArray(pkgLicenses)
    ? pkgLicenses.join(', ')
    : pkgLicenses;

  return (
    <div
      className="flex flex-col justify-center px-4"
      role="listitem"
      style={style}
    >
      <SettingsMenuCard
        className="min-h-12 py-3!"
        key={name}
        title={`${name} ${version ? ` v${version}` : ''}`}
        description={description}
        onClick={() =>
          navigate(routes.licenseDetails, { state: { license, language } })
        }
        trailingIcon="arrow_right"
      />
    </div>
  );
};

type Props = {
  language: 'rust' | 'js';
};

function LicenseList({ language }: Props) {
  const { t } = useTranslation('settings');
  const { rootFontSize, codeDepsJs, codeDepsRust } = useMainState();
  const licenses = language === 'js' ? codeDepsJs : codeDepsRust;
  const rowHeight =
    os === 'linux'
      ? rootFontSize * heightFactorLinux
      : rootFontSize * heightFactor;

  return (
    <PageAnim
      className="xs:max-w-lg h-full flex flex-col"
      data-testid={`license-list-${language}`}
    >
      {licenses.length === 0 ? (
        <span
          className="mt-4 pl-4 italic text-iron dark:text-bombay select-none cursor-default"
          data-testid="license-list-empty"
        >
          {t('legal.emptyData')}
        </span>
      ) : (
        <div className="h-full py-2" data-testid="license-list-container">
          <List
            className="w-full"
            rowHeight={rowHeight}
            rowCount={licenses.length}
            rowComponent={Row}
            rowProps={{ licenses, language }}
            role="list"
            data-testid="license-virtualized-list"
          />
        </div>
      )}
    </PageAnim>
  );
}

export default LicenseList;
