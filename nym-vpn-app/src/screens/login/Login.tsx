import { useMainState } from '../../contexts';
import OGLogin from './OGLogin';
import NewLogin from './new-login/NewLogin';

function Login() {
  const { backendFlags } = useMainState();

  if (backendFlags.privy) {
    return <NewLogin />;
  }

  return <OGLogin />;
}

export default Login;
