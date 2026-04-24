import { useEffect } from 'react';
import { useShallow } from 'zustand/react/shallow';
import { useNavigate } from 'react-router';
import { routes } from '../router';
import { useAppStore } from '../store';

let navigationHandled = false;

function InitialNavigation() {
  const { account, initialized, daemonStatus } = useAppStore(
    useShallow((s) => ({
      account: s.account,
      initialized: s.initialized,
      daemonStatus: s.daemonStatus,
    })),
  );
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
