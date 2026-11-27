import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useLogout } from '../../../hooks';
import { Button, ButtonVariant, ConfirmationDialog } from '../../../ui';

// Log out asks first, as on Android and iOS: it disconnects the VPN and
// removes the account from this device
function LogoutButton({ variant }: { variant: ButtonVariant }) {
  const { t } = useTranslation('settings');
  const { logout, loading } = useLogout();
  const [confirming, setConfirming] = useState(false);

  return (
    <>
      <Button
        variant={variant}
        onClick={() => setConfirming(true)}
        disabled={loading}
        loading={loading}
      >
        {t('account.logout')}
      </Button>
      <ConfirmationDialog
        icon="logout"
        title={t('account.logout-confirm.title')}
        description={t('account.logout-confirm.desc')}
        confirmButtonText={t('account.logout')}
        isOpen={confirming}
        isLoading={loading}
        onConfirm={async () => {
          await logout();
          setConfirming(false);
        }}
        onCancel={() => setConfirming(false)}
      />
    </>
  );
}

export default LogoutButton;
