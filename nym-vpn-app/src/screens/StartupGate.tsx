import { Navigate } from 'react-router';
import { routes } from '../router';
import { useAppStore } from '../store';

let gateDone = false;

function StartupGate() {
  const account = useAppStore((s) => s.account);

  const destination = !gateDone && !account ? routes.onboarding : routes.root;
  // gateDone = true;

  return <Navigate to={destination} replace />;
}

export default StartupGate;
