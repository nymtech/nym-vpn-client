import clsx from 'clsx';
import { useAppStore } from '../store';
import AppError from './AppError';

// Standalone wrapper around `AppError` for the cases where nothing above it is
// mounted: a crash caught by the error boundary, a failed startup sequence, or
// an unhandled rejection with no live UI. `ThemeSetter` lives inside the
// provider tree and cannot be relied on here, so the theme is read straight
// from the store, which is a module-level singleton and defaults to 'light'.
function FatalError({ error }: { error: unknown }) {
  const uiTheme = useAppStore.getState().uiTheme;

  return (
    <div
      className={clsx([uiTheme === 'dark' && 'dark', 'h-full'])}
      data-testid="fatal-error-container"
      data-test-theme={uiTheme}
    >
      <div className="bg-surface-bg h-full">
        <AppError
          error={error}
          onReload={() => {
            window.location.reload();
          }}
        />
      </div>
    </div>
  );
}

export default FatalError;
