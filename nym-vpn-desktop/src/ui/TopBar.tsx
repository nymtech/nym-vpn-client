import { useEffect, useMemo, useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { AppName } from '../constants';
import { routes } from '../router';
import { Routes } from '../types';
import MsIcon from './MsIcon';

type NavLocation = {
  title?: string;
  leftIcon?: string;
  handleLeftNav?: () => void;
  rightIcon?: string;
  handleRightNav?: () => void;
  hidden?: boolean;
};

type NavBarData = {
  [key in Routes]: NavLocation;
};

export default function TopBar() {
  const location = useLocation();
  const navigate = useNavigate();
  const { t } = useTranslation();

  const [currentNavLocation, setCurrentNavLocation] = useState<NavLocation>({
    title: '',
    rightIcon: 'settings',
    handleRightNav: () => {
      navigate(routes.settings);
    },
  });

  const navBarData = useMemo<NavBarData>(() => {
    return {
      '/': {
        title: AppName,
        rightIcon: 'settings',
        handleRightNav: () => {
          navigate(routes.settings);
        },
      },
      '/login': {
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
        hidden: true,
      },
      '/settings': {
        title: t('settings'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/settings/display': {
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
      '/settings/legal/licenses': {
        title: t('legal.licenses', { ns: 'settings' }),
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
      '/entry-node-location': {
        title: t('first-hop-selection'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
      '/exit-node-location': {
        title: t('last-hop-selection'),
        leftIcon: 'arrow_back',
        handleLeftNav: () => {
          navigate(-1);
        },
      },
    };
  }, [t, navigate]);

  useEffect(() => {
    setCurrentNavLocation(navBarData[location.pathname as Routes]);
  }, [location.pathname, navBarData]);

  return (
    <nav
      className={clsx([
        'flex flex-row flex-nowrap justify-between items-center shrink-0',
        'text-baltic-sea dark:text-mercury-pinkish',
        'h-16 text-xl shadow z-50 select-none cursor-default',
        // hide the Topbar on login page
        currentNavLocation.hidden
          ? 'shadow-none dark:bg-baltic-sea bg-blanc-nacre'
          : 'dark:bg-baltic-sea-jaguar bg-white',
      ])}
    >
      {currentNavLocation?.leftIcon ? (
        <button
          className="w-6 mx-4 focus:outline-none"
          onClick={currentNavLocation.handleLeftNav}
        >
          <MsIcon
            icon={currentNavLocation.leftIcon}
            style="dark:text-laughing-jack"
          />
        </button>
      ) : (
        <div className="w-6 mx-4" />
      )}
      {currentNavLocation.title && (
        <p className="justify-self-center tracking-normal">
          {currentNavLocation.title}
        </p>
      )}
      {currentNavLocation?.rightIcon ? (
        <button
          className="w-6 mx-4 focus:outline-none"
          onClick={currentNavLocation.handleRightNav}
        >
          <MsIcon
            icon={currentNavLocation.rightIcon}
            style="dark:text-laughing-jack"
          />
        </button>
      ) : (
        <div className="w-6 mx-4" />
      )}
    </nav>
  );
}
