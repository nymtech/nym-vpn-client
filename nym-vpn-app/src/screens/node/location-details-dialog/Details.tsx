import { Trans, useTranslation } from 'react-i18next';
import { Link, MsIcon } from '../../../ui';
import {
  LocationDetailsArticle,
  QuicSupportArticleUrl,
  ResidentialIpServersUrl,
} from '../../../constants';
import { NodeHop } from '../../../types/index';

function DetailsSection({
  icon,
  title,
  children,
}: {
  icon: string;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col gap-2">
      <div className="text-text-primary flex flex-row items-center gap-2">
        <MsIcon icon={icon} className="text-text-secondary" />
        <h4 className="text-lg">{title}</h4>
      </div>
      <p className="text-text-secondary">{children}</p>
    </div>
  );
}

function LocationSection() {
  const { t } = useTranslation('node-location');
  return (
    <DetailsSection
      icon="location_on"
      title={t('location-details.location.title')}
    >
      <Trans i18nKey="location-details.location.description" ns="node-location">
        Displayed locations are
        <Link
          url={LocationDetailsArticle}
          textClassName="text-black dark:text-white"
        >
          determined from IP addresses
        </Link>
        and may not reflect exact physical locations.
      </Trans>
    </DetailsSection>
  );
}

function QUICSection() {
  const { t } = useTranslation('node-location');
  return (
    <DetailsSection icon="package_2" title={t('location-details.quic.title')}>
      <Trans i18nKey="location-details.quic.description" ns="node-location">
        Improves the Fast mode reliability in restrictive networks by
        <Link
          url={QuicSupportArticleUrl}
          textClassName="text-black dark:text-white"
        >
          wrapping WireGuard traffic in QUIC
        </Link>
        (HTTP/3) to appear as regular web browsing.
      </Trans>
    </DetailsSection>
  );
}

function StreamingSection() {
  const { t } = useTranslation('node-location');
  return (
    <DetailsSection
      icon="smart_display"
      title={t('location-details.streaming.title')}
    >
      <Trans
        i18nKey="location-details.streaming.description"
        ns="node-location"
      >
        <Link
          url={ResidentialIpServersUrl}
          textClassName="text-black dark:text-white"
        >
          Residential IP servers
        </Link>
        optimized for streaming and content access. May experience slower speeds
        due to higher demand and hardware limitations.
      </Trans>
    </DetailsSection>
  );
}

export type Props = {
  node: NodeHop;
};

export function Details({ node }: Props) {
  if (node === 'entry') {
    return (
      <>
        <QUICSection />
        <LocationSection />
      </>
    );
  } else {
    return (
      <>
        <StreamingSection />
        <LocationSection />
      </>
    );
  }
}
