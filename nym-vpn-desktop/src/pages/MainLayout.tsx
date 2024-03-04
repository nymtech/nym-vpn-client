import { Outlet, useLocation } from 'react-router-dom';
import clsx from 'clsx';
import { routes } from '../router';
import { TopBar } from '../ui';

function MainLayout() {
  const location = useLocation();

  return (
    <div
      className={clsx([
        'h-full flex flex-col min-w-80',
        'bg-blanc-nacre text-baltic-sea',
        'dark:bg-baltic-sea dark:text-white',
      ])}
    >
      <TopBar />
      <div
        className={clsx([
          'h-full flex flex-col overflow-auto overscroll-auto p-4',
          location.pathname === routes.licensesRust && '!p-0',
        ])}
      >
        <div className="grow">
          <Outlet />
        </div>
      </div>
    </div>
  );
}

export default MainLayout;
