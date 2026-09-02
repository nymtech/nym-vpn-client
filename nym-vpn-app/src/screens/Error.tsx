import { useRouteError } from 'react-router';
import AppError from './AppError';

// Route-level error element. Rendered inside `MainLayout`'s outlet so the top
// bar survives and the user can still navigate away.
export default function Error() {
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
