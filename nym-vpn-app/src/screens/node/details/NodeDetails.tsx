import React from 'react';
import clsx from 'clsx';
import * as H from 'history';
import { Trans, useTranslation } from 'react-i18next';
import { useLocation, useNavigate } from 'react-router';
import { UiGateway, useMainDispatch } from '../../../contexts';
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
import { uiNodeToRaw } from '../../../contexts/nodes/util';
import { routes } from '../../../router';
import DataCard from './DataCard';

type RouteState = {
  gateway: UiGateway;
  hop: 'entry' | 'exit';
};

function NodeDetails() {
  const dispatch = useMainDispatch() as StateDispatch;
  const location = useLocation() as H.Location<RouteState>;
  const { t } = useTranslation('nodeLocation');
  const navigate = useNavigate();

  const { getCountryName } = useLang();
  const { copy } = useClipboard();
  const { style } = useScore();

  const { gateway, hop } = location.state;
  const { country, exitIpv4, exitIpv6, asn, buildVersion } = gateway;
  const isGoodIp = asn?.type === 'residential';
  const serverLoad = gateway?.wgPerformance?.load;
  const uptime = gateway?.wgPerformance?.uptime24h;
  const lastUpdate = gateway.wgPerformance?.lastUpdatedUtc;
  const asnValue = asn?.asn;
  const asnName = asn?.name;
  const showCard3 = exitIpv4 || exitIpv6 || asnValue || asnName;
  const isSelected =
    gateway.isSelected === 'exit' || gateway.isSelected === 'entry';

  const DataRow = ({
    children,
    label,
  }: {
    children: React.ReactNode;
    label: string;
  }) => (
    <div className="w-full flex justify-between items-center">
      <p className="text-iron dark:text-bombay truncate">{label}</p>
      <div className="flex flex-nowrap items-center gap-2 overflow-hidden">
        {children}
      </div>
    </div>
  );

  const featureRow = (
    label: string,
    feature: string,
    status: 'green' | 'orange' = 'green',
  ) => (
    <DataRow label={label}>
      {status === 'green' ? (
        <MsIcon className="text-malachite text-xl" icon="check" />
      ) : (
        <MsIcon className="text-cheddar text-xl" icon="circle" />
      )}
      <p className="whitespace-nowrap truncate">{feature}</p>
    </DataRow>
  );

  const scoreRow = (label: string, score: Score) => {
    const { icon, color, label: iconLabel } = style(score);

    return (
      <DataRow label={label}>
        <div className="flex gap-1 items-center overflow-hidden">
          <MsIcon className={clsx('text-lg', color)} icon={icon} />
          <p className={clsx('font-medium truncate', color)}>{iconLabel}</p>
        </div>
      </DataRow>
    );
  };

  const IdentityKey = () => (
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
    await kvSet(
      hop === 'entry' ? 'entry-node' : 'exit-node',
      uiNodeToRaw(gateway),
    );
    dispatch({
      type: 'set-node',
      payload: { hop, node: gateway },
    });
    navigate(routes.root);
  };

  console.log('-_-_-_');
  console.log(gateway);
  console.log('-_-_-_');

  return (
    <PageAnim className="xs:max-w-lg h-full flex flex-col mt-2 gap-6 select-none">
      <h1 className="text-lg font-medium dark:text-white">{gateway.name}</h1>
      <div className="flex flex-row items-center gap-2">
        <FlagIcon
          code={country.code.toLowerCase() as countryCode}
          alt={country.code}
          className="h-6"
        />
        <div className="text-lg" data-testid="node-details-country-name">
          {getCountryName(country.code) || country.name}
        </div>
      </div>
      <DataCard>
        {featureRow(
          t('node-details.data.advanced-privacy'),
          t('node-details.data.with-mixnet'),
        )}
        {featureRow(
          t('node-details.data.ip-type'),
          isGoodIp
            ? t('node-details.data.ip-residential')
            : t('node-details.data.ip-datacenter'),
          isGoodIp ? 'green' : 'orange',
        )}
      </DataCard>
      <DataCard
        footer={
          lastUpdate
            ? t('node-details.notes.performance_with_date', {
                date: lastUpdate,
              })
            : t('node-details.notes.performance')
        }
      >
        {scoreRow(
          t('node-details.data.overall-performance'),
          gateway.type === 'wg' ? gateway.wgScore : gateway.mxScore,
        )}
        {serverLoad && scoreRow(t('node-details.data.server-load'), serverLoad)}
        {uptime && (
          <DataRow label={t('node-details.data.uptime')}>
            <p className="font-medium">{`${uptime * 100}%`}</p>
          </DataRow>
        )}
      </DataCard>
      {showCard3 && (
        <DataCard>
          {exitIpv4 && (
            <DataRow label={t('node-details.data.exit-ipv4')}>
              <Link
                text={exitIpv4}
                url={`${IpInfoIoUrl}/${exitIpv4}`}
                color="primary"
                iconClassName="text-lg"
                icon
              />
            </DataRow>
          )}
          {exitIpv6 && (
            <DataRow label={t('node-details.data.exit-ipv6')}>
              <Link
                text={exitIpv6}
                url={`${IpInfoIoUrl}/${exitIpv6}`}
                color="primary"
                iconClassName="text-lg"
                icon
              />
            </DataRow>
          )}
          {asnValue && (
            <DataRow label={t('node-details.data.asn')}>
              <div className="truncate">{asnValue}</div>
            </DataRow>
          )}
          {asnName && (
            <DataRow label={t('node-details.data.asn-name')}>
              <div className="truncate">{asnName}</div>
            </DataRow>
          )}
        </DataCard>
      )}
      <DataCard>
        {buildVersion && (
          <DataRow label={t('node-details.data.build-version')}>
            <div className="truncate">{buildVersion}</div>
          </DataRow>
        )}
        <IdentityKey />
      </DataCard>
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
      {!isSelected && (
        <Button onClick={handleSelect}>
          {t('node-details.select-button')}
        </Button>
      )}
    </PageAnim>
  );
}

export default NodeDetails;
