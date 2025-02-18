import {
  ReactNode,
  isValidElement,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from 'react';
import { useLocation, useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { type } from '@tauri-apps/plugin-os';
import { motion } from 'motion/react';
import { NymVpnTextLogo } from '../assets';
import { useDialog, useMainState } from '../contexts';
import { routes } from '../router';
import { Routes } from '../types';
import MsIcon from './MsIcon';

type NavLocation = {
  title?: string | ReactNode;
  leftIcon?: string;
  handleLeftNav?: () => void;
  rightIcon?: string;
  rightIconClassName?: string;
  handleRightNav?: () => void;
  noBackground?: boolean;
};

type NavBarData = Record<Routes, NavLocation>;

export default function TopBar() {
  const location = useLocation();
  const navigate = useNavigate();
  const { t } = useTranslation();
  const os = type();

  const { uiTheme } = useMainState();
  const { show } = useDialog();

  const [currentNavLocation, setCurrentNavLocation] = useState<NavLocation>({
    title: '',
    rightIcon: 'settings',
    handleRightNav: () => {
      navigate(routes.settings);
    },
  });

  const getMainScreenTitle = useCallback(() => {
    if (os === 'windows' || os === 'macos') {
      // we don't show the logo since the native window-bar already shows it
      return null;
    }
    return (
      <NymVpnTextLogo
        className={clsx(
          'w-24 h-6',
          uiTheme === 'Dark' ? 'fill-white' : 'fill-ash',
        )}
      />
    );
  }, [os, uiTheme]);

  const navBarData = useMemo<NavBarData>(() => {
    return {
      '/': {
        title: getMainScreenTitle(),
        rightIcon: 'settings',
        handleRightNav: () => {
          navigate(routes.settings);
        },
        noBackground: os === 'windows' || os === 'macos',
      },
      '/login': {
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
        noBackground: true,
      },
      '/settings': {
        title: t('settings'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/appearance': {
        title: t('appearance'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/appearance/lang': {
        title: t('language'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/appearance/display': {
        title: t('display-theme'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/logs': {
        title: t('logs'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/feedback': {
        title: t('feedback'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/feedback/send': {
        title: t('feedback'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/legal': {
        title: t('legal'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/legal/licenses-rust': {
        title: t('legal.licenses-rust', { ns: 'settings' }),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/legal/licenses-js': {
        title: t('legal.licenses-js', { ns: 'settings' }),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/legal/license-details': {
        title: t('legal.license', { ns: 'settings' }),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/support': {
        title: t('support'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/dev': {
        title: 'dev',
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/entry-node-location': {
        title: t('first-hop-selection'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
        rightIcon: 'info',
        rightIconClassName:
          'text-cement-feet dark:text-mercury-mist hover:text-gun-powder dark:hover:text-mercury-pinkish',
        handleRightNav: () => {
          show('location-info');
        },
      },
      '/exit-node-location': {
        title: t('last-hop-selection'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
        rightIcon: 'info',
        rightIconClassName:
          'text-cement-feet dark:text-mercury-mist hover:text-gun-powder dark:hover:text-mercury-pinkish',
        handleRightNav: () => {
          show('location-info');
        },
      },
      // these screens do not use the TopBar
      '/hideout': {},
      '/hideout/welcome': {},
    };
  }, [os, t, navigate, getMainScreenTitle, show]);

  useEffect(() => {
    setCurrentNavLocation(navBarData[location.pathname as Routes]);
  }, [location.pathname, navBarData]);

  const renderTitle = (title?: string | ReactNode) => {
    if (typeof title === 'string') {
      return (
        <p className="truncate justify-self-center tracking-normal">
          {currentNavLocation.title}
        </p>
      );
    }
    if (isValidElement(title)) {
      return title;
    }
    return <div></div>;
  };

  return (
    <motion.nav
      initial={{
        opacity: 0,
        scaleX: 0.9,
      }}
      animate={{
        opacity: 1,
        scaleX: 1,
        transition: { duration: 0.2 },
      }}
      className={clsx([
        'flex flex-row flex-nowrap justify-between items-center shrink-0',
        'text-baltic-sea dark:text-mercury-pinkish',
        'h-16 text-xl z-30 select-none cursor-default',
        currentNavLocation.noBackground
          ? 'dark:bg-ash bg-faded-lavender'
          : 'dark:bg-octave-arsenic bg-white',
      ])}
    >
      {currentNavLocation.leftIcon ? (
        <motion.div
          initial={{ translateX: -4, opacity: 0.6 }}
          animate={{ translateX: 0, opacity: 1 }}
          transition={{ duration: 0.15, ease: 'easeOut' }}
        >
          <button
            className="w-6 mx-4 focus:outline-hidden cursor-default"
            onClick={currentNavLocation.handleLeftNav}
          >
            <MsIcon
              icon={currentNavLocation.leftIcon}
              className={clsx([
                'dark:text-laughing-jack transition duration-150',
                'opacity-90 dark:opacity-100 hover:opacity-100 hover:text-black dark:hover:text-white',
              ])}
            />
          </button>
        </motion.div>
      ) : (
        <div className="w-6 mx-4" />
      )}
      {renderTitle(currentNavLocation.title)}
      {currentNavLocation.rightIcon ? (
        <motion.div
          initial={{ translateX: 4, opacity: 0.6 }}
          animate={{ translateX: 0, opacity: 1 }}
          transition={{ duration: 0.15, ease: 'easeOut' }}
        >
          <button
            className="w-6 mx-4 focus:outline-hidden cursor-default"
            onClick={currentNavLocation.handleRightNav}
          >
            <MsIcon
              icon={currentNavLocation.rightIcon}
              className={clsx([
                'dark:text-laughing-jack transition duration-150',
                'opacity-90 dark:opacity-100 hover:opacity-100 hover:text-black dark:hover:text-white',
                currentNavLocation.rightIconClassName,
              ])}
            />
          </button>
        </motion.div>
      ) : (
        <div className="w-6 mx-4" />
      )}
    </motion.nav>
  );
}
