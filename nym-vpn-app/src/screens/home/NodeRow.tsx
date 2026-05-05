import { useCallback, useEffect, useMemo } from 'react';
import { AnimatePresence, motion } from 'motion/react';
import clsx from 'clsx';
import { useTranslation } from 'react-i18next';
import { Button } from '@base-ui/react';
import { useNavigate } from 'react-router';
import { FlagIcon, MsIcon, type countryCode } from '../../ui';
import { useAppStore, useLookupGw } from '../../store';
import { useLang } from '../../hooks';
import {
  Gateway,
  Score,
  SelectedNode,
  isCountry,
  isGateway,
  isRegion,
} from '../../types/index';
import { countriesWithRegions } from '../../constants';
import { QuicTag } from '../index';
import { routes } from '../../router';
import { isBridgeMode, regionToCountryCode } from './util';

import { FoldState } from './NewBottomComponent';
import { ScoreIndicatorContainer } from './ScoreIndicatorContainer';

const DURATION = 0.3;

type NodeRowProps = {
  // label?: string;
  type: 'entry' | 'exit';
  foldState: FoldState;
};

export type SelectedNodeDisplayProps = {
  countryCode?: countryCode;
  name: string;
  location?: string;
  ip?: string;
  showQuic?: boolean;
  disabled?: boolean;
  showStreamOptimized?: boolean;
  showFastest?: boolean;
  score?: Score;
};

export function NodeRow({ type, foldState }: NodeRowProps) {
  const gatewaySelectionAlgorithmConfig = useAppStore(
    (s) => s.gatewaySelectionAlgorithmConfig,
  );

  const state = useAppStore((s) => s.state);

  const navigate = useNavigate();
  const userSelectedNode = useAppStore((s) =>
    type === 'entry' ? s.entryNode : s.exitNode,
  );
  const tunnel = useAppStore((s) => s.tunnel);
  const connectingState = useAppStore((s) => s.connectingState);

  const lookupGw = useLookupGw();

  const label = useMemo(
    () => (type === 'entry' ? 'Nym entry node' : 'Nym exit node'),
    [type],
  );

  const { getCountryName } = useLang();
  const { t } = useTranslation('home');

  const quicConnection =
    isBridgeMode(tunnel?.data) || isBridgeMode(connectingState?.tunnel);
  const quicTag = type === 'entry' && quicConnection;

  const getLocationInfo = useCallback(
    (
      countryCode: string,
      gateway: Gateway | null,
      region?: string,
    ): SelectedNodeDisplayProps => {
      const location = getCountryName(countryCode) || countryCode;
      const locationComponents = [location];
      if (region && region.length > 0) {
        locationComponents.push(region);
      }

      if (gateway) {
        locationComponents.push(gateway.location.city);
      }

      return {
        countryCode: countryCode.toLowerCase() as countryCode,
        name: location,
        location: locationComponents.join(', '),
        ip: gateway?.exitIpv4 || gateway?.exitIpv6 || '',
        showQuic: Boolean(quicTag && gateway?.quic),
        showStreamOptimized:
          type === 'exit' && gateway?.asn?.type === 'residential',
        showFastest: userSelectedNode === 'random' && !gateway?.country?.code,
        score: gateway?.type === 'wg' ? gateway?.wgScore : gateway?.mxScore,
      };
    },
    [getCountryName, userSelectedNode, quicTag, type],
  );

  const getGatewayInfo = useCallback(
    (id: string, gateway: Gateway | null): SelectedNodeDisplayProps => {
      if (!gateway) {
        return {
          name: id,
        };
      }

      const { country, location, name } = gateway;
      const components = [];
      if (location.city.length > 0) {
        components.push(location.city);
      }
      if (
        countriesWithRegions.includes(country.code) &&
        location.region.length > 0
      ) {
        components.push(location.region);
      }
      components.push(getCountryName(country.code) || country.name);

      return {
        countryCode: country.code.toLowerCase() as countryCode,
        name,
        location: components.join(', '),
        ip: gateway?.exitIpv4 || gateway?.exitIpv6 || '',
        showQuic: Boolean(quicTag && gateway?.quic),
        showStreamOptimized:
          type === 'exit' && gateway?.asn?.type === 'residential',
        showFastest: userSelectedNode === 'random' && !gateway?.country?.code,
        score: gateway?.type === 'wg' ? gateway?.wgScore : gateway?.mxScore,
      };
    },
    [getCountryName, userSelectedNode, quicTag, type],
  );

  const nodeData = useCallback(
    (
      selected: SelectedNode,
      gateway: Gateway | null,
    ): SelectedNodeDisplayProps => {
      if (selected === 'random') {
        return {
          name: t('random', { ns: 'common' }),
          location: 'Random server',
          ip: '',
          showQuic: Boolean(quicTag && gateway?.quic),
          showStreamOptimized:
            type === 'exit' && gateway?.asn?.type === 'residential',
          showFastest: userSelectedNode === 'random' && !gateway?.country?.code,
          score: gateway?.type === 'wg' ? gateway?.wgScore : gateway?.mxScore,
        };
      }
      if (isCountry(selected)) {
        return getLocationInfo(selected.country.code, gateway);
      }
      if (isRegion(selected)) {
        return getLocationInfo(
          // TODO handle this better, ie. vpnd should provide country code along with region
          regionToCountryCode(selected.region) || 'US',
          gateway,
          selected.region,
        );
      }
      return getGatewayInfo(selected.gateway.id, gateway);
    },
    [getGatewayInfo, getLocationInfo, userSelectedNode, quicTag, t, type],
  );

  const gateway = useMemo(() => {
    const gw =
      type === 'entry'
        ? tunnel?.entryGwId || connectingState?.entryGwId
        : tunnel?.exitGwId || connectingState?.exitGwId;
    console.log('[NodeRow] gateway2 gw', gw);
    switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
      case 'auto':
        return gw ? lookupGw(gw, type) : null;
      case 'autoEntryExplicitExit':
      case 'explicit':
        if (isGateway(userSelectedNode)) {
          return lookupGw(userSelectedNode.gateway.id, type);
        }
        if (gw) {
          return lookupGw(gw, type);
        }
        return null;
    }
  }, [
    connectingState?.entryGwId,
    connectingState?.exitGwId,
    gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm,
    lookupGw,
    tunnel?.entryGwId,
    tunnel?.exitGwId,
    type,
    userSelectedNode,
  ]);

  const nodeDetails = useMemo(() => {
    switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
      case 'auto':
        return getGatewayInfo(gateway?.id || '', gateway);
      case 'autoEntryExplicitExit':
      case 'explicit':
        return nodeData(userSelectedNode, gateway);
    }
  }, [
    gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm,
    nodeData,
    userSelectedNode,
    gateway,
    getGatewayInfo,
  ]);

  const getTextLabel = () => {
    switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
      case 'auto':
        return nodeDetails.ip ?? 'Best server for my location';
      case 'autoEntryExplicitExit':
        return state === 'connected' ? nodeDetails.ip : nodeDetails.name;
      case 'explicit':
        if (state === 'connected') {
          return gateway?.name ?? nodeDetails.name;
        }
        return nodeDetails.name;
    }
  };

  const getTextDescriptionLabel = () => {
    switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
      case 'auto':
        return nodeDetails.location;
      case 'autoEntryExplicitExit':
      case 'explicit':
        if (isGateway(userSelectedNode)) {
          return nodeDetails.location;
        }
        return state === 'connected' ? nodeDetails.location : null;
    }
  };

  return (
    <>
      {label &&
        gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm !==
          'auto' && (
          <p className="text-secondary text-xs leading-5 tracking-[0.18px]">
            {label}
          </p>
        )}
      <p>noderow type: {type}</p>
      <Button
        onClick={() =>
          navigate(
            type === 'entry'
              ? routes.entryNodeLocation
              : routes.exitNodeLocation,
          )
        }
        className="group relative isolate rounded-xl p-2 w-full"
      >
        {/* Rotating gradient ring on hover — mask center with card bg so only border shows */}
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 z-0 rounded-xl opacity-0 transition-opacity duration-200 ease-out group-hover:opacity-100"
        >
          <div className="absolute inset-0 overflow-hidden rounded-[inherit]">
            {/* Outer: translate only. Inner: rotate only — avoids transform override jump on spin */}
            <div className="absolute left-1/2 top-1/2 size-[260%] -translate-x-1/2 -translate-y-1/2">
              <div
                className={clsx(
                  'size-full will-change-transform backface-hidden',
                  '[background:conic-gradient(from_0deg,var(--color-malachite-200)_0%,var(--color-cornflower)_45%,var(--color-azur)_72%,var(--color-malachite-200)_100%)]',
                  'motion-safe:animate-[spin_3s_linear_infinite]',
                )}
              />
            </div>
          </div>
          <div
            className="absolute inset-[2px] rounded-[calc(0.75rem-2px)] bg-white dark:bg-[#1d1d1f]"
            aria-hidden
          />
        </div>

        <div className="relative z-10 flex flex-col  items-start">
          <div className="flex items-center justify-between gap-4 w-full">
            <div className="flex items-center gap-2 flex-1 overflow-hidden">
              <ScoreIndicatorContainer score={nodeDetails.score} />

              {nodeDetails.countryCode &&
                (state === 'connected' ||
                  gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm !==
                    'auto') && (
                  <FlagIcon
                    code={nodeDetails.countryCode}
                    alt={nodeDetails.name}
                  />
                )}
              <AnimatePresence mode="wait" initial={false}>
                <motion.span
                  key={foldState === 2 ? 'name' : 'ip'}
                  custom={foldState === 2 ? 'name' : 'ip'}
                  variants={{
                    initial: (k: string) => ({
                      opacity: 0,
                      x: k === 'name' ? 14 : -14,
                    }),
                    animate: { opacity: 1, x: 0 },
                    exit: (k: string) => ({
                      opacity: 0,
                      x: k === 'name' ? -14 : 14,
                    }),
                  }}
                  initial="initial"
                  animate="animate"
                  exit="exit"
                  transition={{ duration: DURATION, ease: [0.32, 0.72, 0, 1] }}
                  className="block truncate flex-1 text-start min-w-0 text-baltic-sea dark:text-white text-base leading-6 tracking-[-0.08px] overflow-hidden"
                >
                  {/* {foldState === 2 ? nodeDetails.name : nodeDetails.ip} */}
                  {getTextLabel()}
                </motion.span>
              </AnimatePresence>
            </div>
            <div className="flex flex-row items-center justify-center gap-3">
              {!nodeDetails.showQuic && <QuicTag />}
              {!nodeDetails.showStreamOptimized && (
                <MsIcon icon="smart_display" className="text-cornflower" />
              )}
            </div>
          </div>
          <p className="ml-0 text-secondary text-xs leading-5 tracking-[0.18px]">
            {/* {nodeDetails.location} */}
            {getTextDescriptionLabel()}
            {/* {getTextLabel()} */}
          </p>
        </div>
      </Button>
    </>
  );
}
// export function NodeRow({ type, foldState }: NodeRowProps) {
//   const gatewaySelectionAlgorithmConfig = useAppStore(
//     (s) => s.gatewaySelectionAlgorithmConfig,
//   );

//   const state = useAppStore((s) => s.state);

//   const navigate = useNavigate();
//   const userSelectedNode = useAppStore((s) =>
//     type === 'entry' ? s.entryNode : s.exitNode,
//   );
//   const tunnel = useAppStore((s) => s.tunnel);
//   const connectingState = useAppStore((s) => s.connectingState);
//   const wg = useAppStore((s) => s.wg);

//   // console.log('[NodeRow] type', type);
//   // console.log('[NodeRow] userSelectedNode', userSelectedNode);
//   // console.log('[NodeRow] tunnel', tunnel);
//   // console.log('[NodeRow] connectingState', connectingState);
//   // console.log('[NodeRow] wg', wg);

//   const lookupGw = useLookupGw();

//   // useEffect(() => {
//   //   const gw = tunnel?.exitGwId || connectingState?.exitGwId || undefined;

//   //   for (const country of wg) {
//   //     const gwsearch = country.gateways.find((cg) => cg.id === gw);
//   //     if (gwsearch) {
//   //       console.log('[NodeRow] gwsearch', gwsearch);
//   //     }
//   //   }
//   // }, [wg, tunnel, connectingState]);

//   // const gateway = useMemo(() => {
//   //   // debugger;
//   //   console.log('[NodeRow] useMemo');
//   //   let gw: string | null | undefined = undefined;
//   //   if (type === 'entry') {
//   //     gw = tunnel?.entryGwId || connectingState?.entryGwId;
//   //   } else {
//   //     gw = tunnel?.exitGwId || connectingState?.exitGwId;
//   //   }
//   //   // console.log('[memo][NodeRow] gw', gw);

//   //   let result: Gateway | null = null;

//   //   if (isGateway(userSelectedNode)) {
//   //     result = lookupGw(userSelectedNode.gateway.id, type);
//   //   } else if (gw) {
//   //     result = lookupGw(gw, type);
//   //   }
//   //   // console.log('[NodeRow] result', result);
//   //   return result;
//   // }, [
//   //   connectingState?.entryGwId,
//   //   connectingState?.exitGwId,
//   //   lookupGw,
//   //   userSelectedNode,
//   //   tunnel?.entryGwId,
//   //   tunnel?.exitGwId,
//   //   type,
//   // ]);

//   const gateway = useMemo(() => {
//     const gw =
//       type === 'entry'
//         ? tunnel?.entryGwId || connectingState?.entryGwId
//         : tunnel?.exitGwId || connectingState?.exitGwId;

//     const algo = gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm;

//     // if (!gw) return null;

//     if (
//       gw &&
//       algo === 'auto' &&
//       (state === 'connected' || state === 'connecting')
//     ) {
//       return lookupGw(gw, type);
//     }

//     // console.log('[NodeRow] algo', algo);
//     // console.log('[NodeRow] userSelectedNode', userSelectedNode);
//     if (algo !== 'auto') {
//       if (isGateway(userSelectedNode)) {
//         return lookupGw(userSelectedNode.gateway.id, type);
//       }

//       if (gw) {
//         return lookupGw(gw, type);
//       }

//       // return isGateway(userSelectedNode)
//       //   ? lookupGw(userSelectedNode.gateway.id, type)
//       //   : gw
//       //     ? lookupGw(gw, type)
//       //     : null;
//     }

//     return null;
//   }, [
//     connectingState?.entryGwId,
//     connectingState?.exitGwId,
//     gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm,
//     lookupGw,
//     state,
//     tunnel?.entryGwId,
//     tunnel?.exitGwId,
//     type,
//     userSelectedNode,
//   ]);

//   // console.log('[NodeRow] global gateway', gateway);

//   useEffect(() => {
//     // debugger;
//     const gw = tunnel?.exitGwId || connectingState?.exitGwId || undefined;
//     if (gw) {
//       const result = lookupGw(gw, 'exit');
//       // console.log('[NodeRow] result', result);
//     }
//   }, [lookupGw, tunnel, connectingState]);

//   const label = useMemo(
//     () => (type === 'entry' ? 'Nym entry node' : 'Nym exit node'),
//     [type],
//   );

//   const { getCountryName } = useLang();
//   const { t } = useTranslation('home');

//   const quicConnection =
//     isBridgeMode(tunnel?.data) || isBridgeMode(connectingState?.tunnel);
//   const quicTag = type === 'entry' && quicConnection;

//   const getLocationInfo = useCallback(
//     (
//       countryCode: string,
//       gateway: Gateway | null,
//       region?: string,
//     ): SelectedNodeDisplayProps => {
//       const location = getCountryName(countryCode) || countryCode;
//       const locationComponents = [location];
//       // let subInfo = null;
//       if (region && region.length > 0) {
//         // location = `${location}, ${region}`;
//         locationComponents.push(region);
//       }

//       if (gateway) {
//         locationComponents.push(gateway.location.city);
//       }

//       // if (gateway) {
//       //   const components = [];
//       //   if (gateway.location.city.length > 0) {
//       //     components.push(gateway.location.city);
//       //   }
//       //   if (!region && countriesWithRegions.includes(countryCode)) {
//       //     components.push(gateway.location.region);
//       //   }
//       //   subInfo = `${components.join(', ')} (${gateway.name})`;
//       // }

//       // console.log('[NodeRow] getLocationInfo', {
//       //   countryCode,
//       //   location,
//       //   subInfo,
//       //   quicTag,
//       //   gateway,
//       // });

//       return {
//         countryCode: countryCode.toLowerCase() as countryCode,
//         name: location,
//         // name: gateway?.name || '',
//         location: gateway ? locationComponents.join(', ') : undefined,
//         ip: gateway?.exitIpv4 || gateway?.exitIpv6 || '',
//         showQuic: Boolean(quicTag && gateway?.quic),
//         showStreamOptimized:
//           type === 'exit' && gateway?.asn?.type === 'residential',
//         showFastest: userSelectedNode === 'random' && !gateway?.country?.code,
//         score: gateway?.type === 'wg' ? gateway?.wgScore : gateway?.mxScore,
//       };
//     },
//     [getCountryName, userSelectedNode, quicTag, type],
//   );

//   const getGatewayInfo = useCallback(
//     (id: string, gateway: Gateway | null): SelectedNodeDisplayProps => {
//       if (!gateway) {
//         return {
//           name: id,
//         };
//       }

//       const { country, location, name } = gateway;
//       const components = [];
//       if (location.city.length > 0) {
//         components.push(location.city);
//       }
//       if (
//         countriesWithRegions.includes(country.code) &&
//         location.region.length > 0
//       ) {
//         components.push(location.region);
//       }
//       components.push(getCountryName(country.code) || country.name);

//       return {
//         countryCode: country.code.toLowerCase() as countryCode,
//         name,
//         location: components.join(', '),
//         ip: gateway?.exitIpv4 || gateway?.exitIpv6 || '',
//         showQuic: Boolean(quicTag && gateway?.quic),
//         showStreamOptimized:
//           type === 'exit' && gateway?.asn?.type === 'residential',
//         showFastest: userSelectedNode === 'random' && !gateway?.country?.code,
//         score: gateway?.type === 'wg' ? gateway?.wgScore : gateway?.mxScore,
//       };
//     },
//     [getCountryName, userSelectedNode, quicTag, type],
//   );

//   const nodeData = useCallback(
//     (
//       selected: SelectedNode,
//       gateway: Gateway | null,
//     ): SelectedNodeDisplayProps => {
//       if (selected === 'random') {
//         return {
//           name: t('random', { ns: 'common' }),
//           location: 'Random server',
//           ip: '',
//           showQuic: Boolean(quicTag && gateway?.quic),
//           showStreamOptimized:
//             type === 'exit' && gateway?.asn?.type === 'residential',
//           showFastest: userSelectedNode === 'random' && !gateway?.country?.code,
//           score: gateway?.type === 'wg' ? gateway?.wgScore : gateway?.mxScore,
//         };
//       }
//       if (isCountry(selected)) {
//         return getLocationInfo(selected.country.code, gateway);
//       }
//       if (isRegion(selected)) {
//         return getLocationInfo(
//           // TODO handle this better, ie. vpnd should provide country code along with region
//           regionToCountryCode(selected.region) || 'US',
//           gateway,
//           selected.region,
//         );
//       }
//       return getGatewayInfo(selected.gateway.id, gateway);
//     },
//     [getGatewayInfo, getLocationInfo, userSelectedNode, quicTag, t, type],
//   );

//   // console.log('[NodeRow] gateway2', gateway);
//   // console.log('[NodeRow] nodeData', nodeData(userSelectedNode, gateway));

//   const gateway2 = useMemo(() => {
//     const gw =
//       type === 'entry'
//         ? tunnel?.entryGwId || connectingState?.entryGwId
//         : tunnel?.exitGwId || connectingState?.exitGwId;
//     switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
//       case 'auto':
//         return gw ? lookupGw(gw, type) : null;
//       case 'autoEntryExplicitExit':
//       case 'explicit':
//         if (isGateway(userSelectedNode)) {
//           return lookupGw(userSelectedNode.gateway.id, type);
//         }
//         if (gw) {
//           return lookupGw(gw, type);
//         }
//         return null;
//     }
//   }, [
//     connectingState?.entryGwId,
//     connectingState?.exitGwId,
//     gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm,
//     lookupGw,
//     tunnel?.entryGwId,
//     tunnel?.exitGwId,
//     type,
//     userSelectedNode,
//   ]);

//   const nodeDetails = useMemo(() => {
//     switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
//       case 'auto':
//         console.log('[NodeRow] auto gateway2', gateway2);
//         return getGatewayInfo(gateway2?.id || '', gateway2);
//       case 'autoEntryExplicitExit':
//       case 'explicit':
//         return nodeData(userSelectedNode, gateway2);
//     }
//   }, [
//     gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm,
//     nodeData,
//     userSelectedNode,
//     gateway2,
//     getGatewayInfo,
//   ]);

//   // console.log('[NodeRow] foldState', foldState);
//   console.log('[NodeRow] nodeDetails', nodeDetails);

//   const getTextLabel = () => {
//     switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
//       case 'auto':
//         return nodeDetails.ip ?? 'Best server for my location';
//       case 'autoEntryExplicitExit':
//         return state === 'connected' ? nodeDetails.ip : nodeDetails.name;
//       case 'explicit':
//         return nodeDetails.name ?? 'default name';
//     }
//   };

//   // console.log('[NodeRow] getTextLabel', getTextLabel());

//   const getTextDescriptionLabel = () => {
//     switch (gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm) {
//       case 'auto':
//         return nodeDetails.location;
//       // if (state === 'connected') {
//       //   return nodeDetails.location;
//       // }
//       // return null;
//       // return gateway ? gateway.ip : 'Best server for my location';
//       case 'autoEntryExplicitExit':
//         // return state === 'connected' ? nodeDetails.location : null;
//         return nodeDetails.location;
//       // return 'Nym exit node';
//       case 'explicit':
//         return 'Nym entry node';
//     }
//   };

//   return (
//     <>
//       {label &&
//         gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm !==
//           'auto' && (
//           <p className="text-secondary text-xs leading-5 tracking-[0.18px]">
//             {label}
//           </p>
//         )}
//       <p>noderow type: {type}</p>
//       <Button
//         onClick={() =>
//           navigate(
//             type === 'entry'
//               ? routes.entryNodeLocation
//               : routes.exitNodeLocation,
//           )
//         }
//         className="group relative isolate rounded-xl p-2 w-full"
//       >
//         {/* Rotating gradient ring on hover — mask center with card bg so only border shows */}
//         <div
//           aria-hidden
//           className="pointer-events-none absolute inset-0 z-0 rounded-xl opacity-0 transition-opacity duration-200 ease-out group-hover:opacity-100"
//         >
//           <div className="absolute inset-0 overflow-hidden rounded-[inherit]">
//             {/* Outer: translate only. Inner: rotate only — avoids transform override jump on spin */}
//             <div className="absolute left-1/2 top-1/2 size-[260%] -translate-x-1/2 -translate-y-1/2">
//               <div
//                 className={clsx(
//                   'size-full will-change-transform backface-hidden',
//                   '[background:conic-gradient(from_0deg,var(--color-malachite-200)_0%,var(--color-cornflower)_45%,var(--color-azur)_72%,var(--color-malachite-200)_100%)]',
//                   'motion-safe:animate-[spin_3s_linear_infinite]',
//                 )}
//               />
//             </div>
//           </div>
//           <div
//             className="absolute inset-[2px] rounded-[calc(0.75rem-2px)] bg-white dark:bg-[#1d1d1f]"
//             aria-hidden
//           />
//         </div>

//         <div className="relative z-10 flex flex-col  items-start">
//           <div className="flex items-center justify-between gap-4 w-full">
//             <div className="flex items-center gap-2 flex-1 overflow-hidden">
//               <ScoreIndicatorContainer score={nodeDetails.score} />

//               {nodeDetails.countryCode &&
//                 (state === 'connected' ||
//                   gatewaySelectionAlgorithmConfig.gatewaySelectionAlgorithm !==
//                     'auto') && (
//                   <FlagIcon
//                     code={nodeDetails.countryCode}
//                     alt={nodeDetails.name}
//                   />
//                 )}
//               <AnimatePresence mode="wait" initial={false}>
//                 <motion.span
//                   key={foldState === 2 ? 'name' : 'ip'}
//                   custom={foldState === 2 ? 'name' : 'ip'}
//                   variants={{
//                     initial: (k: string) => ({
//                       opacity: 0,
//                       x: k === 'name' ? 14 : -14,
//                     }),
//                     animate: { opacity: 1, x: 0 },
//                     exit: (k: string) => ({
//                       opacity: 0,
//                       x: k === 'name' ? -14 : 14,
//                     }),
//                   }}
//                   initial="initial"
//                   animate="animate"
//                   exit="exit"
//                   transition={{ duration: DURATION, ease: [0.32, 0.72, 0, 1] }}
//                   className="block truncate flex-1 text-start min-w-0 text-baltic-sea dark:text-white text-base leading-6 tracking-[-0.08px] overflow-hidden"
//                 >
//                   {/* {foldState === 2 ? nodeDetails.name : nodeDetails.ip} */}
//                   {getTextLabel()}
//                 </motion.span>
//               </AnimatePresence>
//             </div>
//             <div className="flex flex-row items-center justify-center gap-3">
//               {!nodeDetails.showQuic && <QuicTag />}
//               {!nodeDetails.showStreamOptimized && (
//                 <MsIcon icon="smart_display" className="text-cornflower" />
//               )}
//             </div>
//           </div>
//           <p className="ml-10 text-secondary text-xs leading-5 tracking-[0.18px]">
//             {/* {nodeDetails.location} */}
//             {getTextDescriptionLabel()}
//             {/* {getTextLabel()} */}
//           </p>
//         </div>
//       </Button>
//     </>
//   );
// }
