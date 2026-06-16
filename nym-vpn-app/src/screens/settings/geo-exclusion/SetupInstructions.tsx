import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { OsType, type } from '@tauri-apps/plugin-os';
import { ButtonIconNew, CardNew, CardNewBody, PageAnim } from '../../../ui';
import { useClipboard } from '../../../hooks';
import { useGeoExclusion } from './utils/useGeoExclusion';

const Platforms = [
  'android',
  'macos',
  'windows',
  'linux',
] as const satisfies readonly OsType[];
type Platform = (typeof Platforms)[number];

type SetupSection = {
  heading?: string;
  intro?: string;
  steps: string[];
  note?: string;
};

function StepList({ steps }: { steps: string[] }) {
  return (
    <CardNew>
      <CardNewBody className="py-2">
        {steps.map((step, index) => (
          <div
            key={step}
            className={clsx(
              'flex items-center gap-4 py-3',
              index !== 0 && 'border-t border-black/8 dark:border-white/10',
            )}
          >
            <span className="bg-brand-primary/15 text-brand-primary flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-sm">
              {index + 1}
            </span>
            <p className="text-text-primary text-sm">{step}</p>
          </div>
        ))}
      </CardNewBody>
    </CardNew>
  );
}

function SetupInstructions() {
  const { copy } = useClipboard();
  const { t } = useTranslation('settings');
  const { listenPort } = useGeoExclusion();
  const [platform, setPlatform] = useState<Platform>(() => {
    const os = type();
    return (Platforms as readonly OsType[]).includes(os)
      ? (os as Platform)
      : 'windows';
  });

  const sections = t(`geo-exclusion.setup-instructions.sections.${platform}`, {
    returnObjects: true,
    port: listenPort,
  }) as unknown as SetupSection[];

  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-6 select-none">
      <div className="bg-surface-elev flex gap-1 rounded-2xl p-1">
        {Platforms.map((p) => (
          <button
            key={p}
            type="button"
            onClick={() => setPlatform(p)}
            className={clsx(
              'flex-1 rounded-xl px-3 py-2 text-sm transition',
              platform === p
                ? 'bg-brand-primary text-surface-bg'
                : 'text-text-secondary hover:bg-surface-hair',
            )}
          >
            {t(`geo-exclusion.setup-instructions.platforms.${p}`)}
          </button>
        ))}
      </div>

      {sections.map((section) => (
        <div
          key={section.heading ?? section.intro}
          className="flex flex-col gap-3"
        >
          {(section.heading || section.intro) && (
            <div className="flex flex-col gap-1">
              {section.heading && (
                <h2 className="text-text-primary text-base font-medium">
                  {section.heading}
                </h2>
              )}
              {section.intro && (
                <p className="text-text-secondary text-sm">{section.intro}</p>
              )}
            </div>
          )}

          <StepList steps={section.steps} />

          {section.note && (
            <p className="text-text-secondary text-xs">{section.note}</p>
          )}
        </div>
      ))}

      <div className="flex flex-col gap-1">
        <p className="text-text-secondary text-xs uppercase">
          {t('geo-exclusion.setup-instructions.proxy-address')}
        </p>
        <div className="flex items-center gap-2">
          <p className="text-text-primary font-mono">{`127.0.0.1:${listenPort}`}</p>
          <ButtonIconNew
            icon="content_copy"
            onClick={() => copy(`127.0.0.1:${listenPort}`)}
            clickFeedback
            size="small"
          />
        </div>
      </div>
    </PageAnim>
  );
}

export default SetupInstructions;
