import { CSSProperties, useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { FixedSizeList as List } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';
import { useMainState } from '../../../../contexts';
import { routes } from '../../../../router';
import { PageAnim, SettingsMenuCard } from '../../../../ui';
import { CodeDependency } from '../../../../types';

const heightFactor = 8;

const Row = ({
  style,
  license: { name, version, licenses },
  license,
  language,
}: {
  style: CSSProperties;
  license: CodeDependency;
  language: 'js' | 'rust';
}) => {
  const navigate = useNavigate();

  return (
    <div className="flex flex-col justify-center px-4" style={style}>
      <SettingsMenuCard
        className="min-h-12 !py-3"
        key={name}
        title={`${name} ${version ? ` v${version}` : ''}`}
        desc={Array.isArray(licenses) ? licenses.join(', ') : licenses}
        onClick={async () =>
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
  const [licenses, setLicenses] = useState<CodeDependency[]>([]);
  const [itemSize, setItemSize] = useState<number>(400);

  const { t } = useTranslation('settings');
  const { rootFontSize, codeDepsJs, codeDepsRust } = useMainState();

  useEffect(() => {
    if (language === 'js') {
      setLicenses(codeDepsJs);
    }
    if (language === 'rust') {
      setLicenses(codeDepsRust);
    }
  }, [language, codeDepsJs, codeDepsRust]);

  useEffect(() => {
    setItemSize(rootFontSize * heightFactor);
  }, [rootFontSize]);

  return (
    <PageAnim className="h-full flex flex-col">
      {licenses.length === 0 ? (
        <span className="mt-4 pl-4 italic text-dim-gray dark:text-mercury-mist select-none cursor-default">
          {t('legal.emptyData')}
        </span>
      ) : (
        <div className="h-full py-2">
          <AutoSizer disableWidth>
            {({ height }) => (
              <List
                className="w-full"
                height={height}
                itemCount={licenses.length}
                width="100%"
                itemSize={itemSize}
              >
                {({ index, style }) => (
                  <Row
                    style={style}
                    license={licenses[index]}
                    language={language}
                  />
                )}
              </List>
            )}
          </AutoSizer>
        </div>
      )}
    </PageAnim>
  );
}

export default LicenseList;
