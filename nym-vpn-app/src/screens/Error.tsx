import { useRouteError } from 'react-router';
import AppError from './AppError';

// Route-level error element. Rendered inside `MainLayout`'s outlet so the top
// bar survives and the user can still navigate away.
export default function Error() {
  // `useRouteError` is typed unknown for good reason: the thrown value can be
  // an Error, a Response, a string or null, so it must not be destructured
  const error = useRouteError();

  return (
    <AppError
      error={error}
      onReload={() => {
        window.location.reload();
      }}
    />
  );
}
