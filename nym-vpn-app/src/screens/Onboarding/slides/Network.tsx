import { Trans, useTranslation } from 'react-i18next';
import { DvpnRing, MixnetRing } from '../../../assets/onboarding';
import type { NetworkVariant } from '../VariantToggle';

const RINGS = {
  dvpn: DvpnRing,
  mixnet: MixnetRing,
} as const;

function Network({ variant }: { variant: NetworkVariant }) {
  const { t } = useTranslation('onboarding');
  const Ring = RINGS[variant];

  return (
    <div className="flex flex-col items-center gap-8 px-4">
      <h1 className="text-text-primary text-center text-2xl uppercase">
        {t(`network.${variant}.title`)}
      </h1>
      <Ring className="h-full max-h-44 w-auto max-w-full shrink-0" />
      <p className="text-text-secondary text-center text-sm whitespace-pre-line">
        <Trans
          i18nKey={`network.${variant}.description`}
          ns="onboarding"
          components={{ bold: <strong /> }}
        />
      </p>
    </div>
  );
}

export default Network;
