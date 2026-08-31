import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { OsType, type } from '@tauri-apps/plugin-os';
import { CardNew, CardNewBody, PageAnim } from '../../../ui';
import { useGeoExclusion } from './utils/useGeoExclusion';

const Platforms = [
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
  const { t } = useTranslation('settings');
  const { listenPort } = useGeoExclusion();
  const os = type();
  const platform: Platform = (Platforms as readonly OsType[]).includes(os)
    ? (os as Platform)
    : 'windows';

  const sections = t(`geo-exclusion.setup-instructions.sections.${platform}`, {
    returnObjects: true,
    port: listenPort,
  }) as unknown as SetupSection[];

  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-6 select-none">
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
    </PageAnim>
  );
}

export default SetupInstructions;
