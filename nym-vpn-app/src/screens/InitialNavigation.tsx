import { useEffect } from 'react';
import { useNavigate } from 'react-router';
import { useMainState } from '../contexts';
import { routes } from '../router';

let navigationHandled = false;

function InitialNavigation() {
  const { account, initialized } = useMainState();
  const navigate = useNavigate();

  useEffect(() => {
    if (!initialized || navigationHandled) {
      return;
    }

    // prevent multiple navigations
    navigationHandled = true;

    if (!account) {
      navigate(routes.onboarding, { replace: true });
    } else {
      navigate(routes.root, { replace: true });
    }
  }, [account, initialized, navigate]);

  return null;
}

export default InitialNavigation;
