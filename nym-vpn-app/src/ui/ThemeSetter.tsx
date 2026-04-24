import React from 'react';
import clsx from 'clsx';
import { useAppStore } from '../store';

export default function ThemeSetter({
  children,
}: {
  children: React.ReactNode;
}) {
  const uiTheme = useAppStore((s) => s.uiTheme);

  return (
    <div
      className={clsx([uiTheme === 'dark' && 'dark', 'h-full'])}
      data-testid="theme-setter"
      data-test-theme={uiTheme}
    >
      {children}
    </div>
  );
}
