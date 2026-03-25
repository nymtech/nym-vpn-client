import { useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import PageAnim from '../../../ui/PageAnim';
import SettingsMenuCard from '../../../ui/SettingsMenuCard';
import Switch from '../../../ui/Switch';
import { useDialog } from '../../../contexts';
import InfoDialog from './InfoDialog';
import AppItem, { AppEntry } from './AppItem';

// Icon background colors for app entries (derived from app name hash)
const ICON_COLORS = [
  '#e53935',
  '#e67c00',
  '#f9a825',
  '#43a047',
  '#00897b',
  '#1e88e5',
  '#6d4c41',
  '#8e24aa',
  '#546e7a',
  '#d81b60',
];

function iconColorForName(name: string): string {
  let hash = 0;
  for (let i = 0; i < name.length; i++) {
    hash = (hash * 31 + name.charCodeAt(i)) >>> 0;
  }
  return ICON_COLORS[hash % ICON_COLORS.length];
}

// Mock app data – will be replaced with data from the backend
const MOCK_APPS: AppEntry[] = [
  'Adobe Acrobat',
  'AirWater',
  'Amazon',
  'Brave Browser',
  'Chrome',
  'Coinbase',
  'Discord',
  'Dropbox',
  'Firefox',
  'Finder',
  'Google Drive',
  'Gmail',
  'Notion',
  'Notes',
  'Safari',
  'Signal',
  'Slack',
  'Spotify',
  'Terminal',
  'Telegram',
  'Visual Studio Code',
  'VLC',
  'WhatsApp',
  'Xcode',
  'YouTube',
  'Zoom',
].map((name) => ({
  id: name.toLowerCase().replace(/\s+/g, '-'),
  name,
  iconColor: iconColorForName(name),
  state: 'included' as const,
}));

function SplitTunneling() {
  const { t } = useTranslation('settings');
  const { isOpen, close } = useDialog();
  const [enabled, setEnabled] = useState(false);
  const [apps, setApps] = useState<AppEntry[]>(MOCK_APPS);
  const sectionRefs = useRef<Record<string, HTMLDivElement | null>>({});

  const groupedApps = useMemo(() => {
    const groups: Record<string, AppEntry[]> = {};
    apps.forEach((app) => {
      const letter = app.name[0].toUpperCase();
      if (!groups[letter]) groups[letter] = [];
      groups[letter].push(app);
    });
    return groups;
  }, [apps]);

  const letters = useMemo(() => Object.keys(groupedApps).sort(), [groupedApps]);

  const handleStateChange = (id: string, state: 'excluded' | 'included') => {
    setApps((prev) =>
      prev.map((app) => (app.id === id ? { ...app, state } : app)),
    );
  };

  const scrollToSection = (letter: string) => {
    sectionRefs.current[letter]?.scrollIntoView({ behavior: 'smooth' });
  };

  return (
    <PageAnim className="flex flex-col mt-2 gap-4">
      <InfoDialog
        isOpen={isOpen('split-tunneling-info')}
        onClose={() => close('split-tunneling-info')}
      />

      {/* Enable split tunneling */}
      <SettingsMenuCard
        title={t('split-tunneling.enable')}
        leadingIcon="call_split"
        trailingComponent={<Switch checked={enabled} onChange={setEnabled} />}
        onClick={() => setEnabled((v) => !v)}
      />

      {/* Description */}
      <p className="text-sm text-iron dark:text-bombay">
        {t('split-tunneling.description')}
      </p>

      {/* Apps section */}
      <div className="flex flex-col gap-2">
        <p className="text-base font-semibold text-baltic-sea dark:text-white select-none">
          {t('split-tunneling.apps')}
        </p>

        {/* App list with alphabetical sidebar */}
        <div className="flex items-stretch gap-0">
          {/* App list */}
          <div className="flex-1 min-w-0 rounded-lg overflow-hidden">
            {letters.map((letter) => (
              <div
                key={letter}
                ref={(el) => {
                  sectionRefs.current[letter] = el;
                }}
              >
                {/* Section divider */}
                <div className="px-4 py-1 bg-mercury/40 dark:bg-mine-shaft/60">
                  <span className="text-xs text-iron dark:text-bombay select-none">
                    {letter}
                  </span>
                </div>

                {/* Apps in this section */}
                {groupedApps[letter].map((app, i) => (
                  <div key={app.id}>
                    <AppItem app={app} onStateChange={handleStateChange} />
                    {i < groupedApps[letter].length - 1 && (
                      <div className="mx-4 h-px bg-mercury/60 dark:bg-white/5" />
                    )}
                  </div>
                ))}
              </div>
            ))}
          </div>

          {/* Alphabetical sidebar */}
          <div className="sticky top-0 gap-1.5 self-start flex flex-col items-center justify-between w-5 ml-3.5">
            {letters.map((letter) => (
              <button
                key={letter}
                className={clsx(
                  'text-xs h-4 w-full text-center cursor-default select-none',
                  'text-iron dark:text-bombay hover:text-baltic-sea dark:hover:text-white',
                  'transition-noborder',
                )}
                onClick={() => scrollToSection(letter)}
              >
                {letter}
              </button>
            ))}
          </div>
        </div>
      </div>
    </PageAnim>
  );
}

export default SplitTunneling;
