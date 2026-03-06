import { useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useMainState } from '../contexts';
import { routes } from '../router';

let navigationHandled = false;

function InitialNavigation() {
  const { account, initialized, daemonStatus } = useMainState();
  const navigate = useNavigate();

  useEffect(() => {
    if (!initialized || navigationHandled) {
      return;
    }

    // prevent multiple navigations
    navigationHandled = true;

    if (!account) {
      navigate(routes.onboarding, { replace: true });
    }
  }, [account, daemonStatus, initialized, navigate]);

  return null;
}

export default InitialNavigation;
