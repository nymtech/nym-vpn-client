import { MouseEvent } from 'react';
import { DialogTitle } from '@headlessui/react';
import { useTranslation } from 'react-i18next';
import { Button, Dialog, Link, MsIcon } from '../../ui';
import { PrivacyPolicyUrl } from '../../constants';

export type TelemetryInfoKind = 'connection' | 'monitoring';

type InfoButtonProps = {
  kind: TelemetryInfoKind;
  label: string;
  onOpen: (kind: TelemetryInfoKind) => void;
  'data-testid'?: string;
};

// The (i) button shown next to a telemetry toggle label; opens the dialog below.
// Stops propagation so it doesn't also toggle the row it sits in.
export function InfoButton({ kind, label, onOpen, ...rest }: InfoButtonProps) {
  const { t } = useTranslation('welcome');
  return (
    <button
      type="button"
      aria-label={t('info-aria', { name: label })}
      onClick={(e: MouseEvent) => {
        e.stopPropagation();
        onOpen(kind);
      }}
      className="text-text-tertiary hover:text-text-secondary flex items-center"
      data-testid={rest['data-testid']}
    >
      <MsIcon icon="info" className="text-lg!" />
    </button>
  );
}

type Props = {
  kind: TelemetryInfoKind | null;
  onClose: () => void;
};

// Shared "What's shared" dialog for the telemetry opt-in toggles, reused by the
// onboarding screen and the Data & privacy settings screen. Content is keyed by
// `kind` in the `welcome` i18n namespace.
function TelemetryInfoDialog({ kind, onClose }: Props) {
  const { t } = useTranslation('welcome');

  if (!kind) {
    return null;
  }

  const shares = t(`${kind}.shares`, { returnObjects: true }) as string[];

  return (
    <Dialog open onClose={onClose} data-testid="telemetry-info-dialog">
      <div className="flex flex-col items-center gap-3">
        <MsIcon icon="info" className="text-text-primary" />
        <DialogTitle as="h3" className="text-text-primary text-center text-xl">
          {t('popup.title')}
        </DialogTitle>
      </div>

      <ul className="text-text-secondary mt-4 flex list-disc flex-col gap-2 pl-5 text-sm">
        {shares.map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>

      <p className="text-text-secondary border-surface-bg mt-4 border-t pt-3 text-xs">
        <span className="text-text-primary font-semibold">
          {t('popup.never-label')}
        </span>{' '}
        {t(`${kind}.never`)}
      </p>
      <p className="text-text-tertiary mt-2 text-xs">{t(`${kind}.handling`)}</p>

      <div className="mt-6 flex flex-col items-center gap-3">
        <Button onClick={onClose} className="w-full">
          {t('popup.got-it')}
        </Button>
        <Link
          color="iron"
          text={t('privacy-statement-link')}
          url={PrivacyPolicyUrl}
        />
      </div>
    </Dialog>
  );
}

export default TelemetryInfoDialog;
