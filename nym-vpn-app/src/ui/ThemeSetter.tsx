import React from 'react';
import clsx from 'clsx';
import { useMainState } from '../contexts';

export default function ThemeSetter({
  children,
}: {
  children: React.ReactNode;
}) {
  const { uiTheme } = useMainState();

  return (
    <div
      className={clsx([uiTheme === 'dark' && 'dark', 'h-full'])}
      data-testid="theme-setter"
      data-theme={uiTheme}
    >
      {children}
    </div>
  );
}
