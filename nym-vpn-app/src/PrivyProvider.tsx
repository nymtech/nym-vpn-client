import { mainnet } from '@privy-io/chains';
import { PrivyProvider } from '@privy-io/react-auth';
import { useMainState } from './contexts';

export const MyPrivyProvider = ({
  children,
}: {
  children: React.ReactNode;
}) => {
  const { uiTheme } = useMainState();

  return (
    <PrivyProvider
      appId={import.meta.env.VITE_PRIVY_APP_ID}
      config={{
        loginMethods: ['google', 'twitter', 'github'],
        embeddedWallets: {
          ethereum: {
            createOnLogin: 'all-users',
          },
        },
        appearance: {
          theme: uiTheme,
          logo: '/icon.svg',
          accentColor: '#14e76f',
          walletChainType: 'ethereum-only',
        },
        intl: {
          defaultCountry: 'US',
        },
        supportedChains: [mainnet],
      }}
    >
      {children}
    </PrivyProvider>
  );
};
