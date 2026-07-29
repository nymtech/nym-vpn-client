import { useMemo } from 'react';
import clsx from 'clsx';
import dayjs from 'dayjs';
import * as H from 'history';
import { Trans, useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { useLocation, useNavigate } from 'react-router';
import { useShallow } from 'zustand/react/shallow';
import { isSelectedNodeType } from '../../../types/node';
import { Gateway } from '../../../types/tauri';
import { favoriteKey, nodeToFavorite } from '../../../types/favorites';
import { useNodeListState } from '../../../store/nodeListState';
import { useFavorites } from '../../../store/favoritesState';
import FavoriteStar from '../FavoriteStar';
import {
  Button,
  ButtonIconNew,
  CardDataRow,
  CardDivider,
  CardNew,
  CardNewBody,
  CardNewFooter,
  CardNewHeader,
  FlagIcon,
  Link,
  MsIcon,
  PageAnim,
  countryCode,
} from '../../../ui';
import { useClipboard, useLang, useScore, useToast } from '../../../hooks';
import {
  IpInfoIoUrl,
  NetworkExplorerNodeUrl,
  SupportServerLocationUrl,
} from '../../../constants';
import { routes } from '../../../router';
import { dispatch, useAppStore } from '../../../store';
import { ScoreIndicator } from '../ScoreIndicator';
import { LewesIcon } from '../../../assets/index';

type RouteState = {
  gateway: Gateway;
  hop: 'entry' | 'exit';
};

function NodeDetails() {
  const {
    backendFlags,
    entryNode,
    exitNode,
    quic: quicSetting,
    algoConfig,
  } = useAppStore(
    useShallow((s) => ({
      backendFlags: s.backendFlags,
      entryNode: s.entryNode,
      exitNode: s.exitNode,
      quic: s.quic,
      algoConfig: s.gatewaySelectionAlgorithmConfig,
    })),
  );
  const location = useLocation() as H.Location<RouteState>;
  const { t } = useTranslation('node-location');
  const navigate = useNavigate();

  const { add } = useToast();

  const { getCountryName } = useLang();
  const { copy } = useClipboard();
  const { performance: getPerformance, serverLoad: getLoad } = useScore();
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

  const favorites = useFavorites(hop);
  const favorite = useMemo(
    () => nodeToFavorite({ nodeType: 'gateway', id: gateway.id }),
    [gateway.id],
  );
  const isFavorite = useMemo(() => {
    const key = favoriteKey(favorite);
    return favorites.some((f) => favoriteKey(f) === key);
  }, [favorites, favorite]);
  const selectedNode = isSelectedNodeType(gateway, entryNode, exitNode);
  const isSelected = selectedNode === 'exit' || selectedNode === 'entry';
  const quic = backendFlags.quic && gateway.quic;
  const overallScore =
    gateway.type === 'wg' ? gateway.wgScore : gateway.mxScore;

  const performance = getPerformance(overallScore);
  const serverLoadStyle = useMemo(
    () => (serverLoad ? getLoad(serverLoad) : null),
    [serverLoad, getLoad],
  );

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

  const handleSelect = async () => {
    if (isSelected) return;

    const node = { gateway: { id: gateway.id } };
    // Mirror of Node.tsx: picking an exit while in 'auto' flips us into
    // 'autoEntryExplicitExit'. Apply the algorithm change first so a failure
    // aborts before set_node diverges from the daemon; roll back if set_node
    // later fails.
    const needsAlgoFlip =
      hop === 'exit' && algoConfig.gatewaySelectionAlgorithm === 'auto';
    if (needsAlgoFlip) {
      try {
        await invoke('set_gateway_selection_algorithm', {
          algorithm: 'autoEntryExplicitExit',
        });
        dispatch({
          type: 'set-gateway-selection-algorithm-config',
          config: {
            ...algoConfig,
            gatewaySelectionAlgorithm: 'autoEntryExplicitExit',
          },
        });
      } catch (error: unknown) {
        console.error(
          'failed to set gateway selection algorithm to [autoEntryExplicitExit]',
          error,
        );
        add({
          id: 'node-select-error',
          title: t('node-details.error.title'),
          description: t('node-details.error.description'),
          type: 'error',
        });
        return;
      }
    }

    try {
      await invoke('set_node', {
        node,
        hop,
      });
      dispatch({
        type: 'set-node',
        payload: { hop, node },
      });
    } catch (error: unknown) {
      console.error('failed to select node', error);
      add({
        id: 'node-select-error',
        title: t('node-details.error.title'),
        description: t('node-details.error.description'),
        type: 'error',
      });
      if (needsAlgoFlip) {
        try {
          await invoke('set_gateway_selection_algorithm', {
            algorithm: 'auto',
          });
          dispatch({
            type: 'set-gateway-selection-algorithm-config',
            config: { ...algoConfig, gatewaySelectionAlgorithm: 'auto' },
          });
        } catch (rollbackError: unknown) {
          console.error(
            'failed to rollback gateway selection algorithm to [auto]',
            rollbackError,
          );
        }
      }
      return;
    }
    navigate(routes.root);
    resetSaved(hop);
  };

  return (
    <PageAnim className="flex h-full cursor-default flex-col">
      <div className="min-h-0 grow overflow-auto">
        <div className="flex flex-col gap-4 p-4">
          {/* Card 1: Server info */}
          <CardNew>
            <CardNewHeader>
              <FlagIcon
                code={country.code.toLowerCase() as countryCode}
                alt={country.code}
                className="h-6 w-6 shrink-0 rounded-full"
              />
              <p className="text-text-primary ml-4 truncate text-base">
                {gateway.name}
              </p>
              <FavoriteStar
                favorite={favorite}
                isFavorite={isFavorite}
                hop={hop}
                className="ml-auto"
              />
            </CardNewHeader>
            <CardDivider />
            <CardNewBody className="flex-col gap-3 py-4">
              <p className="text-text-primary text-sm font-medium underline dark:text-white">
                {serverLocation()}
              </p>
              {gateway.description && (
                <p className="text-text-secondary text-sm">
                  {gateway.description}
                </p>
              )}
            </CardNewBody>
          </CardNew>

          {/* Card 2: Node features */}
          <CardNew>
            <CardNewHeader>
              <p className="text-text-primary text-sm dark:text-white">
                {t('node-details.data.node-features')}
              </p>
            </CardNewHeader>
            <CardNewBody className="pb-4">
              {/* Advanced privacy */}
              <CardDataRow label={t('node-details.data.advanced-privacy')}>
                <MsIcon
                  icon="visibility_off"
                  className="text-brand-primary text-xl"
                />
                <p className="text-text-primary whitespace-nowrap dark:text-white">
                  {t('node-details.data.with-mixnet')}
                </p>
              </CardDataRow>
              <CardDivider />
              {/* Streaming & IP */}
              <CardDataRow label={t('node-details.data.ip-type')}>
                <MsIcon
                  icon={isGoodIp ? 'smart_display' : 'dns'}
                  className={clsx(
                    'text-xl',
                    isGoodIp ? 'text-status-info' : 'text-text-secondary',
                  )}
                />
                <p className="text-text-primary whitespace-nowrap dark:text-white">
                  {isGoodIp
                    ? t('node-details.data.ip-residential')
                    : t('node-details.data.ip-datacenter')}
                </p>
              </CardDataRow>
              <CardDivider />
              {/* Post-quantum secure keys */}
              <CardDataRow label={t('node-details.data.lewes-protocol-label')}>
                <LewesIcon className="text-text-secondary text-xl" />
                <p className="text-text-primary whitespace-nowrap dark:text-white">
                  {t('node-details.data.lewes-protocol')}
                </p>
              </CardDataRow>
              {/* Anti-censorship */}
              {backendFlags.quic && (
                <>
                  <CardDivider />
                  <CardDataRow label={t('node-details.data.anti-censorship')}>
                    <MsIcon
                      filled
                      icon={quic ? 'package_2' : 'circle'}
                      className="text-text-secondary text-xl"
                    />
                    <p className="text-text-primary whitespace-nowrap dark:text-white">
                      {quic
                        ? t('node-details.data.quic-protocol')
                        : t('node-details.data.standard-protocol')}
                    </p>
                  </CardDataRow>
                </>
              )}
              {gateway.nodeFamilyName && (
                <>
                  <CardDivider />
                  <CardDataRow label="Node family">
                    <p className="text-text-primary whitespace-nowrap dark:text-white">
                      {gateway.nodeFamilyName}
                    </p>
                  </CardDataRow>
                </>
              )}
            </CardNewBody>
            {backendFlags.quic && !quicSetting && (
              <CardNewFooter>
                <p className="text-text-secondary text-xs">
                  <Trans
                    i18nKey="node-details.notes.anti-censorship"
                    ns="node-location"
                  >
                    <Link
                      className="text-status-info! underline"
                      to={routes.antiCensorship}
                    >
                      Enable &quot;QUIC protocol&quot;
                    </Link>
                    in Anti-censorship Settings to use this feature
                  </Trans>
                </p>
              </CardNewFooter>
            )}
          </CardNew>

          {/* Card 3: Performance metrics */}
          <CardNew>
            <CardNewHeader>
              <p className="text-text-primary text-sm dark:text-white">
                {t('node-details.data.performance-metrics')}
              </p>
            </CardNewHeader>
            <CardNewBody className="pb-4">
              {/* Overall performance */}
              <CardDataRow label={t('node-details.data.overall-performance')}>
                <div className="flex items-center gap-1 select-none">
                  <ScoreIndicator score={overallScore} />
                  <p
                    className={clsx('truncate font-medium', performance.color)}
                  >
                    {performance.label}
                  </p>
                </div>
              </CardDataRow>
              {/* Server load */}
              {serverLoad && (
                <>
                  <CardDivider />
                  <CardDataRow label={t('node-details.data.server-load')}>
                    <p
                      className={clsx(
                        'truncate font-medium select-none',
                        serverLoadStyle?.color,
                      )}
                    >
                      {serverLoadStyle?.label}
                    </p>
                  </CardDataRow>
                </>
              )}
              {/* Uptime */}
              {uptime !== undefined && (
                <>
                  <CardDivider />
                  <CardDataRow label={t('node-details.data.uptime')}>
                    <p className="font-medium select-none">
                      {Math.round(uptime * 100)}%
                    </p>
                  </CardDataRow>
                </>
              )}
            </CardNewBody>
            <CardNewFooter>
              <p className="text-text-secondary text-xs whitespace-pre-line">
                {lastUpdate
                  ? t('node-details.notes.performance_with_time', {
                      relativeTime: dayjs().to(dayjs(lastUpdate)),
                    })
                  : t('node-details.notes.performance')}
              </p>
            </CardNewFooter>
          </CardNew>

          {/* Card 4: Connection details */}
          {showCard3 && (
            <CardNew>
              <CardNewHeader>
                <p className="text-text-primary text-sm dark:text-white">
                  {t('node-details.data.connection-details')}
                </p>
              </CardNewHeader>
              <CardNewBody className="pb-4">
                {exitIpv4 && (
                  <CardDataRow label={t('node-details.data.exit-ipv4')}>
                    <Link
                      text={exitIpv4}
                      url={`${IpInfoIoUrl}/${exitIpv4}`}
                      color="primary"
                      iconClassName="text-lg"
                      icon
                      selectable
                    />
                  </CardDataRow>
                )}

                {exitIpv6 && (
                  <>
                    <CardDivider />
                    <CardDataRow label={t('node-details.data.exit-ipv6')}>
                      <Link
                        text={exitIpv6}
                        url={`${IpInfoIoUrl}/${exitIpv6}`}
                        color="primary"
                        iconClassName="text-lg"
                        icon
                        selectable
                      />
                    </CardDataRow>
                  </>
                )}

                {asnValue && (
                  <>
                    <CardDivider />
                    <CardDataRow label={t('node-details.data.asn')}>
                      <p className="text-text-primary truncate dark:text-white">
                        {asnValue}
                      </p>
                    </CardDataRow>
                  </>
                )}

                {asnName && (
                  <>
                    <CardDivider />
                    <CardDataRow label={t('node-details.data.asn-name')}>
                      <p className="text-text-primary truncate dark:text-white">
                        {asnName}
                      </p>
                    </CardDataRow>
                  </>
                )}
              </CardNewBody>
            </CardNew>
          )}

          {/* Card 5: Build information */}
          <CardNew>
            <CardNewHeader>
              <p className="text-text-primary text-sm dark:text-white">
                {t('node-details.data.build-information')}
              </p>
            </CardNewHeader>
            <CardNewBody className="pb-4">
              {buildVersion && (
                <>
                  <CardDataRow label={t('node-details.data.build-version')}>
                    <p className="text-text-primary truncate dark:text-white">
                      {buildVersion}
                    </p>
                  </CardDataRow>
                  <CardDivider />
                </>
              )}
              <div className="flex w-full flex-col gap-2 py-[7px]">
                <p className="text-text-secondary text-sm">
                  {t('node-details.data.identity-key')}
                </p>
                <div className="flex items-center justify-between gap-3">
                  <p className="text-text-primary flex-1 overflow-hidden font-mono text-xs text-wrap wrap-break-word">
                    {gateway.id}
                  </p>
                  <ButtonIconNew
                    size="small"
                    icon="content_copy"
                    onClick={() => copy(gateway.id, false)}
                    clickFeedback
                  />
                </div>
              </div>
            </CardNewBody>
          </CardNew>

          {/* Footer links */}
          <div className="flex flex-col gap-4 px-1 pb-2">
            <Link
              text={t('node-details.links.missing-info')}
              url={SupportServerLocationUrl}
              icon
              color="cornflower"
            />
            <p className="text-text-secondary">
              <Trans
                i18nKey="node-details.links.explorer"
                ns="node-location"
                components={{
                  1: (
                    <Link
                      url={`${NetworkExplorerNodeUrl}/${gateway.id}`}
                      color="cornflower"
                      icon
                    />
                  ),
                }}
              />
            </p>
          </div>
        </div>
      </div>

      {!isSelected && (
        <div className="p-4">
          <Button onClick={handleSelect}>
            {t('node-details.select-button')}
          </Button>
        </div>
      )}
    </PageAnim>
  );
}

export default NodeDetails;
