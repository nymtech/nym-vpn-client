import { useEffect, useState } from 'react';
import { FixedSizeList as List } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';
import licensesUrl from '/licenses-js.json?url';
import { useMainState } from '../../../contexts';

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
type License = {
  name: string;
} & LicenseInfo;

const heightFactor = 32;

const Row = ({
  style,
  license: {
    name,
    licenses,
    repository,
    publisher,
    email,
    licenseText,
    copyright,
  },
}: {
  style: React.CSSProperties;
  license: License;
}) => {
  return (
    <article
      key={name}
      className="flex flex-col gap-2 overflow-y-hidden mb-4 py-4 border-b pl-4 border-mercury-pinkish dark:border-gun-powder"
      style={style}
    >
      <h2 className="text-lg font-bold overflow-ellipsis">{name}</h2>
      {licenses && (
        <div className="font-bold">
          {Array.isArray(licenses) ? licenses.join(', ') : licenses}
        </div>
      )}
      {repository && (
        <a
          className="text-sm break-words hover:underline"
          href={repository}
          target="_blank"
          rel="noreferrer"
        >
          {repository}
        </a>
      )}
      {publisher && <span>{publisher}</span>}
      {email && (
        <a
          className="text-sm break-words hover:underline"
          href={`mailto:${email}`}
        >
          {email}
        </a>
      )}
      {licenseText && (
        <div className="text-sm break-words mr-6 my-2 overflow-scroll text-gun-powder dark:text-mercury-mist">
          {licenseText}
        </div>
      )}
      {copyright && (
        <div className="text-sm italic break-words pr-4 text-gun-powder dark:text-mercury-mist">
          {copyright}
        </div>
      )}
    </article>
  );
};

function LicensesJs() {
  const [licenses, setLicenses] = useState<License[]>([]);
  const [itemSize, setItemSize] = useState<number>(400);

  const { rootFontSize } = useMainState();

  useEffect(() => {
    fetch(licensesUrl)
      .then((response) => response.json())
      .then((data: LicensesJson) => {
        const list = Object.entries(data).map(([name, info]) => {
          return { name, ...info };
        });
        setLicenses(list);
      })
      .catch((e) => {
        console.warn('Failed to fetch licenses data', e);
      });
  }, []);

  useEffect(() => {
    setItemSize(rootFontSize * heightFactor);
  }, [rootFontSize]);

  return (
    <div className="h-full flex flex-col">
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
              <Row style={style} license={licenses[index]} />
            )}
          </List>
        )}
      </AutoSizer>
    </div>
  );
}

export default LicensesJs;
