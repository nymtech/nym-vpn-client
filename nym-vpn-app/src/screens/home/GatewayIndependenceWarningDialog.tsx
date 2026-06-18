import { DialogTitle } from '@headlessui/react';
import { Trans, useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { Button, Dialog, MsIcon } from '../../ui';
import { routes } from '../../router';
import { useGwIndependenceWarning } from '../../contexts/gatewayIndependence';

function GatewayIndependenceWarningDialog() {
  const { t } = useTranslation('home');
  const navigate = useNavigate();
  const { isOpen, accept, cancel } = useGwIndependenceWarning();

  return (
    <Dialog
      open={isOpen}
      onClose={cancel}
      className="flex flex-row gap-4"
      data-testid="gw-independence-warning-dialog"
    >
      <div className="flex flex-row gap-4">
        <MsIcon icon="warning" filled className="text-brand-primary shrink-0" />
        <div className="flex flex-col gap-4">
          <DialogTitle as="h3" className="text-text-primary w-full text-xl">
            {t('gateway-independence-warning.title')}
          </DialogTitle>
          <p className="text-text-secondary italic">
            <Trans
              t={t}
              i18nKey="gateway-independence-warning.disable-reminders"
              components={{
                settingsLink: (
                  <button
                    type="button"
                    className="text-brand-primary underline"
                    onClick={() => {
                      cancel();
                      navigate(routes.notifications);
                    }}
                  />
                ),
              }}
            />
          </p>
          <div className="flex w-full gap-3">
            <Button
              variant="outlined"
              onClick={cancel}
              className="p w-auto!"
              data-testid="gw-independence-cancel"
            >
              {t('gateway-independence-warning.cancel')}
            </Button>
            <Button
              onClick={accept}
              className="w-auto!"
              data-testid="gw-independence-confirm"
            >
              {t('gateway-independence-warning.connect-anyway')}
            </Button>
          </div>
        </div>
      </div>
    </Dialog>
  );
}

export default GatewayIndependenceWarningDialog;
