import { ReactNode, isValidElement, useEffect, useMemo, useState } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { motion } from 'motion/react';
import { NymVpnTextLogo } from '../assets';
import { useDialog, useTopBar } from '../contexts';
import { routes } from '../router';
import { Routes } from '../types';
import { ActionMenu } from '../screens';
import { useSystemTheme } from '../state';
import { useAppStore } from '../store';
import { ButtonIconNew } from './ButtonIcon';
import { StaggeredText } from './StaggeredText';

type NavLocation = {
  title?: string | ReactNode;
  leftIcon?: string;
  handleLeftNav?: () => void;
  rightIcon?: string;
  rightComponent?: ReactNode;
  rightIconClassName?: string;
  handleRightNav?: () => void;
  noBackground?: boolean;
};

type NavBarData = Record<Routes, NavLocation>;

export default function TopBar() {
  const location = useLocation();
  const navigate = useNavigate();
  const { t } = useTranslation();

  const uiTheme = useAppStore((s) => s.uiTheme);
  const { show } = useDialog();
  const { customLeftNavHandler } = useTopBar();

  const { handleThemeChange } = useSystemTheme();

  const [currentNavLocation, setCurrentNavLocation] = useState<NavLocation>({
    title: '',
    rightIcon: 'settings',
    handleRightNav: () => {
      navigate(routes.settings);
    },
  });

  const navBarData = useMemo<NavBarData>(() => {
    return {
      '/login-screen2': {
        title: (
          <NymVpnTextLogo
            className={clsx(
              'w-24 h-6',
              uiTheme === 'dark' ? 'fill-white' : 'fill-ash',
            )}
          />
        ),
        leftIcon: uiTheme === 'dark' ? 'dark_mode' : 'light_mode',
        handleLeftNav: () => {
          // handleThemeChange(uiTheme === 'dark' ? 'light' : 'dark');
          navigate(-1);
        },
        rightIcon: 'settings',
        handleRightNav: () => {
          navigate(routes.settings);
        },
        noBackground: true,
      },
      '/': {
        title: (
          <NymVpnTextLogo
            className={clsx(
              'w-24 h-6',
              uiTheme === 'dark' ? 'fill-white' : 'fill-ash',
            )}
          />
        ),
        leftIcon: uiTheme === 'dark' ? 'dark_mode' : 'light_mode',
        handleLeftNav: () => {
          handleThemeChange(uiTheme === 'dark' ? 'light' : 'dark');
        },
        rightIcon: 'settings',
        handleRightNav: () => {
          navigate(routes.settings);
        },
        noBackground: true,
      },
      '/signup': {
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
        noBackground: true,
      },
      '/login': {
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
        noBackground: true,
      },
      '/onboarding': {
        rightIcon: 'close',
        handleRightNav: () => navigate(routes.root),
        noBackground: true,
      },
      '/settings': {
        title: t('settings'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/account': {
        title: t('account.title', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/appearance': {
        title: t('appearance'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/appearance/lang': {
        title: t('language'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/appearance/display': {
        title: t('display-theme'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/data-privacy': {
        title: t('privacy.title', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/data-privacy/logs': {
        title: t('logs'),
        leftIcon: 'keyboard_arrow_left',
        rightComponent: <ActionMenu />,
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/data-privacy/diagnostic': {
        title: t('diagnostic.title', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/dns': {
        title: t('dns.topbar-title', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/mixnet-tuning': {
        title: t('mixnet-tuning.title', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/split-tunneling': {
        title: t('split-tunneling.title', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
        rightIcon: 'info',
        handleRightNav: () => {
          show('split-tunneling-info');
        },
      },
      '/settings/anti-censorship': {
        title: t('anti-censorship.title', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/socks5': {
        title: t('app-proxy.title', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/feedback': {
        title: t('feedback'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/feedback/send': {
        title: t('feedback'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/legal': {
        title: t('legal'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/legal/licenses-rust': {
        title: t('legal.licenses-rust', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/legal/licenses-js': {
        title: t('legal.licenses-js', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/legal/license-details': {
        title: t('legal.license', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/support': {
        title: t('support'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/dev': {
        title: 'dev',
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/entry-node-location': {
        title: t('first-hop-selection'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
        rightIcon: 'info',
        rightIconClassName:
          'text-iron dark:text-bombay hover:text-baltic-sea dark:hover:text-white',
        handleRightNav: () => {
          show('location-info');
        },
      },
      '/exit-node-location': {
        title: t('last-hop-selection'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
        rightIcon: 'info',
        rightIconClassName:
          'text-iron dark:text-bombay hover:text-baltic-sea dark:hover:text-white',
        handleRightNav: () => {
          show('location-info');
        },
      },
      '/node-details': {
        title: t('server-details'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/account/select-a-plan': {
        title: (
          <NymVpnTextLogo
            className={clsx(
              'w-24 h-6',
              uiTheme === 'dark' ? 'fill-white' : 'fill-ash',
            )}
          />
        ),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
        noBackground: true,
      },
      // these screens do not use the TopBar
      '/hideout': {},
      '/hideout/welcome': {},
      // TODO
      '/account': {},
    };
  }, [t, navigate, show, uiTheme, handleThemeChange]);

  useEffect(() => {
    setCurrentNavLocation(navBarData[location.pathname as Routes]);
  }, [location.pathname, navBarData]);

  const defaultLeftNavHandler = () => {
    navigate(-1);
  };

  const renderTitle = (title?: string | ReactNode) => {
    if (typeof title === 'string') {
      return (
        <StaggeredText
          text={title}
          className="truncate justify-self-center tracking-normal"
          data-testid="top-bar-title-text"
        />
      );
    }
    if (isValidElement(title)) {
      return title;
    }
    return <div data-testid="top-bar-title-empty"></div>;
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
        'text-baltic-sea dark:text-white',
        'h-16 text-xl z-30 select-none cursor-default',
        'px-4 py-2',
        currentNavLocation.noBackground
          ? 'dark:bg-aph bg-gray'
          : 'dark:bg-[#1E1E1E] bg-white',
      ])}
      data-testid="top-bar"
      data-test-route={location.pathname}
      data-test-no-background={
        currentNavLocation.noBackground ? 'true' : 'false'
      }
    >
      {currentNavLocation.leftIcon ? (
        <motion.div
          initial={{ translateX: -4, opacity: 0.6 }}
          animate={{ translateX: 0, opacity: 1 }}
          transition={{ duration: 0.15, ease: 'easeOut' }}
          data-testid="top-bar-left-button-container"
        >
          <ButtonIconNew
            icon={currentNavLocation.leftIcon}
            onClick={
              customLeftNavHandler ??
              currentNavLocation.handleLeftNav ??
              defaultLeftNavHandler
            }
          />
        </motion.div>
      ) : (
        <div className="w-6 mx-4" data-testid="top-bar-left-spacer" />
      )}
      <div data-testid="top-bar-title-container" className="text-xl">
        {renderTitle(currentNavLocation.title)}
      </div>
      {currentNavLocation.rightIcon || currentNavLocation.rightComponent ? (
        <motion.div
          initial={{ translateX: 4, opacity: 0.6 }}
          animate={{ translateX: 0, opacity: 1 }}
          transition={{ duration: 0.15, ease: 'easeOut' }}
          data-testid="top-bar-right-button-container"
        >
          {currentNavLocation.rightComponent &&
            currentNavLocation.rightComponent}
          {currentNavLocation.rightIcon && (
            <ButtonIconNew
              icon={currentNavLocation.rightIcon}
              onClick={currentNavLocation.handleRightNav!}
            />
          )}
        </motion.div>
      ) : (
        <div className="w-6 mx-4" data-testid="top-bar-right-spacer" />
      )}
    </motion.nav>
  );
}
