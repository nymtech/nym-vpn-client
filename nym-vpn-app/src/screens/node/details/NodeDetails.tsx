import React from 'react';
import clsx from 'clsx';
import dayjs from 'dayjs';
import * as H from 'history';
import { Trans, useTranslation } from 'react-i18next';
import { useLocation, useNavigate } from 'react-router';
import {
  UiGateway,
  useMainDispatch,
  useMainState,
  useNodeListState,
} from '../../../contexts';
import {
  Button,
  ButtonIcon,
  FlagIcon,
  Link,
  MsIcon,
  PageAnim,
  countryCode,
} from '../../../ui';
import { useClipboard, useLang, useScore } from '../../../hooks';
import { Score, StateDispatch } from '../../../types';
import {
  IpInfoIoUrl,
  NetworkExplorerNodeUrl,
  SupportServerLocationUrl,
} from '../../../constants';
import { kvSet } from '../../../kvStore';
import { uiNodeToSelectedNode } from '../../../contexts/node-list/util';
import { routes } from '../../../router';
import DataCard from './DataCard';

type RouteState = {
  gateway: UiGateway;
  hop: 'entry' | 'exit';
};

function NodeDetails() {
  const { backendFlags } = useMainState();
  const dispatch = useMainDispatch() as StateDispatch;
  const location = useLocation() as H.Location<RouteState>;
  const { t } = useTranslation('nodeLocation');
  const navigate = useNavigate();

  const { getCountryName } = useLang();
  const { copy } = useClipboard();
  const { performance, serverLoad: serverLoadStyle } = useScore();
  const { reset: resetSaved } = useNodeListState();

  const { gateway, hop } = location.state;
  const {
    country,
    exitIpv4,
    exitIpv6,
    asn,
    buildVersion,
    location: gwLocation,
  } = gateway;
  const isGoodIp = asn?.type === 'residential';
  const serverLoad = gateway?.wgPerformance?.load;
  const uptime = gateway?.wgPerformance?.uptime24h;
  const lastUpdate = gateway.wgPerformance?.lastUpdatedUtc;
  const asnValue = asn?.asn;
  const asnName = asn?.name;
  const showCard3 = exitIpv4 || exitIpv6 || asnValue || asnName;
  const isSelected =
    gateway.isSelected === 'exit' || gateway.isSelected === 'entry';
  const quic = backendFlags.quic && gateway.quic;

  console.log('gateway', gateway);
  // debugger;

  const DataRow = ({
    children,
    label,
  }: {
    children: React.ReactNode;
    label: string;
  }) => (
    <div className="w-full flex justify-between items-center">
      <p className="text-iron dark:text-bombay truncate select-none">{label}</p>
      <div className="flex flex-nowrap items-center gap-2 overflow-hidden">
        {children}
      </div>
    </div>
  );

  const featureRow = (
    label: string,
    feature: string,
    icon: React.ReactNode,
    status: 'green' | 'orange' = 'green',
  ) => (
    <DataRow label={label}>
      {status === 'green' ? (
        icon
      ) : (
        <MsIcon
          className="dark:text-king-nacho text-cheddar text-xl"
          icon="circle"
        />
      )}
      <p className="whitespace-nowrap truncate">{feature}</p>
    </DataRow>
  );

  const scoreRow = (label: string, score: Score) => {
    const { icon, color, label: iconLabel } = performance(score);

    return (
      <DataRow label={label}>
        <div className="flex gap-1 items-center overflow-hidden select-none">
          <MsIcon className={clsx('text-lg', color)} icon={icon} />
          <p className={clsx('font-medium truncate', color)}>{iconLabel}</p>
        </div>
      </DataRow>
    );
  };

  const serverLoadRow = (label: string, score: Score) => {
    const { color, label: iconLabel } = serverLoadStyle(score);

    return (
      <DataRow label={label}>
        <p className={clsx('font-medium truncate select-none', color)}>
          {iconLabel}
        </p>
      </DataRow>
    );
  };

  const identityKey = (
    <div className="w-full flex flex-col gap-2">
      <p className="text-iron dark:text-bombay truncate">
        {t('node-details.data.identity-key')}
      </p>
      <div className="flex justify-between gap-3 break-words">
        <p className="font-mono text-sm flex-wrap text-wrap break-words overflow-hidden">
          {gateway.id}
        </p>
        <ButtonIcon
          className="self-start"
          iconClassName="!text-xl"
          clickedIconClassName="!text-xl"
          icon="content_copy"
          color="chalk"
          onClick={() => copy(gateway.id, false)}
          clickFeedback
          noDefaultSize
        />
      </div>
    </div>
  );

  const handleSelect = async () => {
    if (isSelected) {
      return;
    }
    const selectedNode = uiNodeToSelectedNode(gateway);
    await kvSet(hop === 'entry' ? 'entry-node' : 'exit-node', selectedNode);
    dispatch({
      type: 'set-node',
      payload: { hop, node: selectedNode },
    });
    navigate(routes.root);
    resetSaved(hop);
  };

  const card1 = [
    {
      row: featureRow(
        t('node-details.data.advanced-privacy'),
        t('node-details.data.with-mixnet'),
        <MsIcon
          icon="visibility_off"
          className="dark:text-malachite text-malachite-moss text-xl"
        />,
      ),
      key: 'privacy',
    },
    {
      row: featureRow(
        t('node-details.data.ip-type'),
        isGoodIp
          ? t('node-details.data.ip-residential')
          : t('node-details.data.ip-datacenter'),
        <MsIcon icon="smart_display" className="text-cornflower text-xl" />,
        isGoodIp ? 'green' : 'orange',
      ),
      key: 'ip-type',
    },
    backendFlags.quic && {
      row: featureRow(
        t('node-details.data.anti-censorship'),
        quic
          ? t('node-details.data.quic-protocol')
          : t('node-details.data.standard-protocol'),
        <MsIcon icon="package_2" className="text-azur text-xl" />,
        quic ? 'green' : 'orange',
      ),
      key: 'anticensor-protocal',
    },
  ];
  const card2 = [
    {
      row: scoreRow(
        t('node-details.data.overall-performance'),
        gateway.type === 'wg' ? gateway.wgScore : gateway.mxScore,
      ),
      key: 'overall-perf',
    },
    serverLoad && {
      row: serverLoadRow(t('node-details.data.server-load'), serverLoad),
      key: 'load-score',
    },
    uptime !== undefined && {
      row: (
        <DataRow label={t('node-details.data.uptime')}>
          <p className="font-medium">{`${uptime * 100}%`}</p>
        </DataRow>
      ),
      key: 'uptime',
    },
  ];
  const card3 = [
    exitIpv4 && {
      row: (
        <DataRow label={t('node-details.data.exit-ipv4')}>
          <Link
            text={exitIpv4}
            url={`${IpInfoIoUrl}/${exitIpv4}`}
            color="primary"
            iconClassName="text-lg"
            icon
            selectable
          />
        </DataRow>
      ),
      key: 'exitIpv4',
    },
    exitIpv6 && {
      row: (
        <DataRow label={t('node-details.data.exit-ipv6')}>
          <Link
            text={exitIpv6}
            url={`${IpInfoIoUrl}/${exitIpv6}`}
            color="primary"
            textClassName="select-text"
            iconClassName="text-lg"
            icon
            selectable
          />
        </DataRow>
      ),
      key: 'exitIpv6',
    },
    asnValue && {
      row: (
        <DataRow label={t('node-details.data.asn')}>
          <div className="truncate">{asnValue}</div>
        </DataRow>
      ),
      key: 'asn-value',
    },
    asnName && {
      row: (
        <DataRow label={t('node-details.data.asn-name')}>
          <div className="truncate">{asnName}</div>
        </DataRow>
      ),
      key: 'asn-name',
    },
  ];
  const card4 = [
    buildVersion && {
      row: (
        <DataRow label={t('node-details.data.build-version')}>
          <div className="truncate">{buildVersion}</div>
        </DataRow>
      ),
      key: 'build-version',
    },
    { row: identityKey, key: 'id-key' },
  ];

  const Card1Footer = (
    <p className="text-iron dark:text-bombay">
      <Trans i18nKey="node-details.notes.anti-censorship" ns="nodeLocation">
        <Link className="text-black dark:text-white" to={routes.antiCensorship}>
          Enable “QUIC protocol”
        </Link>
        in Anti-censorship Settings to use this feature
      </Trans>
    </p>
  );

  const card2Footer = lastUpdate
    ? t('node-details.notes.performance_with_time', {
        relativeTime: dayjs().to(dayjs(lastUpdate)),
      })
    : t('node-details.notes.performance');

  const serverLocation = () => {
    const components = [];
    if (gwLocation.city.length > 0) {
      components.push(gwLocation.city);
    }
    if (gwLocation.region.length > 0) {
      components.push(gwLocation.region);
    }
    components.push(getCountryName(country.code) || country.name);
    return components.join(', ');
  };

  return (
    <PageAnim className="xs:max-w-lg h-full flex flex-col cursor-default">
      <div className="flex-1 overflow-auto flex flex-col gap-6 p-4">
        <h1 className="text-lg font-medium dark:text-white break-words">
          {gateway.name}
        </h1>
        <div className="flex flex-row items-center gap-2 select-none">
          <FlagIcon
            code={country.code.toLowerCase() as countryCode}
            alt={country.code}
            className="h-6"
          />
          <div className="text-lg">{serverLocation()}</div>
        </div>
        {gateway.description && (
          <p className="text-iron dark:text-bombay">{gateway.description}</p>
        )}
        <DataCard
          rows={card1}
          footer={hop === 'entry' && quic && Card1Footer}
        />
        <DataCard rows={card2} footer={card2Footer} />
        {showCard3 && <DataCard rows={card3} />}
        <DataCard rows={card4} />
        <div className="flex flex-col gap-2 select-none">
          <Link
            text={t('node-details.links.missing-info')}
            url={SupportServerLocationUrl}
            className="text-baltic-sea dark:text-white"
            iconClassName="text-lg"
            color="iron"
            icon
          />
          <p className="text-iron dark:text-bombay">
            <Trans
              i18nKey="node-details.links.explorer"
              ns="nodeLocation"
              components={{
                networkExplorerLink: (
                  <Link
                    text="Network Explorer"
                    url={`${NetworkExplorerNodeUrl}/${gateway.id}`}
                    color="primary"
                    iconClassName="text-lg"
                    icon
                  />
                ),
              }}
            />
          </p>
        </div>
      </div>
      {!isSelected && (
        <div className="p-4 bg-white dark:bg-charcoal border-t border-bombay dark:border-iron">
          <Button onClick={handleSelect}>
            {t('node-details.select-button')}
          </Button>
        </div>
      )}
    </PageAnim>
  );
}

export default NodeDetails;
