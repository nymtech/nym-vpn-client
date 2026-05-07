import { Navigate } from 'react-router';
import { routes } from '../router';
import { useAppStore } from '../store';

function StartupGate() {
  const account = useAppStore((s) => s.account);

  const destination = !account ? routes.onboarding : routes.root;

  return <Navigate to={destination} replace />;
}

export default StartupGate;
