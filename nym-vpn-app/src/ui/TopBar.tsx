import { ReactNode, isValidElement, useEffect, useMemo, useState } from 'react';
import { useLocation, useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { motion } from 'motion/react';
import { NymVpnTextLogo } from '../assets';
import { useDialog, useTopBar } from '../contexts';
import { routes } from '../router';
import { Routes, UiTheme } from '../types';
import { useSetProfile } from '../hooks';
import { ActionMenu } from '../screens';
import ProfilesMenu from '../screens/home/ProfilesMenu';
import { useSystemTheme } from '../state';
import { useAppStore } from '../store';
import BetaPill from './BetaPill';
import { ButtonIconNew } from './ButtonIcon';
import { StaggeredText } from './StaggeredText';

type NavLocation = {
  title?: string | ReactNode;
  leftIcon?: string;
  leftComponent?: ReactNode;
  handleLeftNav?: () => void;
  rightIcon?: string;
  rightComponent?: ReactNode;
  rightIconClassName?: string;
  handleRightNav?: () => void;
  noBackground?: boolean;
};

type NavBarData = Record<Routes, NavLocation>;

function TopNymLogo({ uiTheme }: { uiTheme: UiTheme }) {
  return (
    <NymVpnTextLogo
      className={clsx(
        'h-6 w-24',
        uiTheme === 'dark' ? 'fill-white' : 'fill-text-primary',
      )}
    />
  );
}

export default function TopBar() {
  const location = useLocation();
  const navigate = useNavigate();
  const { t } = useTranslation();

  const uiTheme = useAppStore((s) => s.uiTheme);
  const { show } = useDialog();
  const { customLeftNavHandler } = useTopBar();

  const { handleThemeChange } = useSystemTheme();
  const setProfile = useSetProfile();

  const [currentNavLocation, setCurrentNavLocation] = useState<NavLocation>({
    title: '',
    rightIcon: 'settings',
    handleRightNav: () => {
      navigate(routes.settings);
    },
  });

  const navBarData = useMemo<NavBarData>(() => {
    const backNav = (title: NavLocation['title']): NavLocation => ({
      title,
      leftIcon: 'keyboard_arrow_left',
      handleLeftNav: () => {
        navigate(-1);
      },
    });

    return {
      '/technical-optin': {
        leftIcon: uiTheme === 'dark' ? 'dark_mode' : 'light_mode',
        handleLeftNav: () =>
          handleThemeChange(uiTheme === 'dark' ? 'light' : 'dark'),
        rightIcon: 'settings',
        handleRightNav: () => {
          navigate(routes.settings);
        },
        noBackground: true,
      },
      '/welcome': {
        title: <TopNymLogo uiTheme={uiTheme} />,
        leftIcon: uiTheme === 'dark' ? 'dark_mode' : 'light_mode',
        handleLeftNav: () =>
          handleThemeChange(uiTheme === 'dark' ? 'light' : 'dark'),
        rightIcon: 'settings',
        handleRightNav: () => {
          navigate(routes.settings);
        },
        noBackground: true,
      },
      '/home': {
        title: <TopNymLogo uiTheme={uiTheme} />,
        leftComponent: <ProfilesMenu onSelect={setProfile} />,
        rightIcon: 'settings',
        handleRightNav: () => {
          navigate(routes.settings);
        },
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
      '/settings/geo-exclusion': {
        title: (
          <span className="flex items-center gap-2">
            {t('geo-exclusion.title', { ns: 'settings' })}
            <BetaPill />
          </span>
        ),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/geo-exclusion/setup-instructions': {
        title: t('geo-exclusion.setup-instructions.title', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/geo-exclusion/select-region': backNav(
        t('geo-exclusion.select-region.title', { ns: 'settings' }),
      ),
      '/settings/anti-censorship': {
        title: t('anti-censorship.title', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/profiles': backNav(t('profiles.title')),
      '/settings/socks5': {
        title: t('app-proxy.title', { ns: 'settings' }),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/notifications': {
        title: t('notifications.title', { ns: 'settings' }),
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
          'text-text-secondary hover:text-text-primary dark:hover:text-white',
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
          'text-text-secondary hover:text-text-primary dark:hover:text-white',
        handleRightNav: () => {
          show('location-info');
        },
      },
      '/node-location': {
        title: t('location'),
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
        rightIcon: 'info',
        rightIconClassName:
          'text-text-secondary hover:text-text-primary dark:hover:text-white',
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
        title: <TopNymLogo uiTheme={uiTheme} />,
        leftIcon: 'keyboard_arrow_left',
        handleLeftNav: () => {
          navigate(-1);
        },
        noBackground: true,
      },
      // these screens do not use the TopBar
      '/hideout': {},
      '/hideout/onboarding': {},
      // TODO
      '/account': {},
    };
  }, [t, navigate, show, uiTheme, handleThemeChange, setProfile]);

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
          className="justify-self-center truncate tracking-normal"
          data-testid="top-bar-title-text"
        />
      );
    }
    if (isValidElement(title)) {
      return title;
    }
    return <div data-testid="top-bar-title-empty"></div>;
  };

  const {
    title,
    leftIcon,
    leftComponent,
    handleLeftNav,
    rightIcon,
    rightComponent,
    rightIconClassName,
    handleRightNav,
    noBackground,
  } = currentNavLocation;

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
        'flex shrink-0 flex-row flex-nowrap items-center justify-between',
        'text-text-primary',
        'z-30 h-16 cursor-default text-xl select-none',
        'px-4 py-2',
        noBackground ? 'bg-surface-bg' : 'bg-surface-elev',
      ])}
      data-testid="top-bar"
      data-test-route={location.pathname}
      data-test-no-background={noBackground ? 'true' : 'false'}
    >
      {leftIcon || leftComponent ? (
        <motion.div
          initial={{ translateX: -4, opacity: 0.6 }}
          animate={{ translateX: 0, opacity: 1 }}
          transition={{ duration: 0.15, ease: 'easeOut' }}
          data-testid="top-bar-left-button-container"
        >
          {leftComponent}
          {leftIcon && (
            <ButtonIconNew
              icon={leftIcon}
              onClick={
                customLeftNavHandler ?? handleLeftNav ?? defaultLeftNavHandler
              }
            />
          )}
        </motion.div>
      ) : (
        <div className="mx-4 w-6" data-testid="top-bar-left-spacer" />
      )}
      <div data-testid="top-bar-title-container" className="text-xl">
        {renderTitle(title)}
      </div>
      {rightIcon || rightComponent ? (
        <motion.div
          initial={{ translateX: 4, opacity: 0.6 }}
          animate={{ translateX: 0, opacity: 1 }}
          transition={{ duration: 0.15, ease: 'easeOut' }}
          data-testid="top-bar-right-button-container"
        >
          {rightComponent}
          {rightIcon && (
            <ButtonIconNew
              icon={rightIcon}
              onClick={handleRightNav!}
              className={rightIconClassName}
            />
          )}
        </motion.div>
      ) : (
        <div className="mx-4 w-6" data-testid="top-bar-right-spacer" />
      )}
    </motion.nav>
  );
}
