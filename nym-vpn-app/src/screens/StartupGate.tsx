import { Navigate } from 'react-router';
import { routes } from '../router';
import { useAppStore } from '../store';

function StartupGate() {
  const daemonStatus = useAppStore((s) => s.daemonStatus);
  const account = useAppStore((s) => s.account);

  if (daemonStatus === 'auth-denied' || daemonStatus === 'down') {
    return <Navigate to={routes.root} replace />;
  }

  const destination = !account ? routes.onboarding : routes.root;

  return <Navigate to={destination} replace />;
}

export default StartupGate;
