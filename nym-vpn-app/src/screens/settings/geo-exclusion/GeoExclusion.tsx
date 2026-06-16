import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import { routes } from '../../../router';
import {
  CardHeaderSwitch,
  CardNew,
  InfoBanner,
  MsIcon,
  PageAnim,
} from '../../../ui';
import SettingsGroup from '../SettingsGroup';
import { useGeoExclusion } from './utils/useGeoExclusion';
import { ExcludedRegions, Socks5PortCard } from './components';

function GeoExclusion() {
  const { t } = useTranslation('settings');
  const navigate = useNavigate();
  const { enabled, listenPort, excludedCountries, setEnabled, setPort } =
    useGeoExclusion();

  const handleToggle = () => {
    setEnabled(!enabled);
  };

  return (
    <PageAnim className="mt-2 flex h-full flex-col gap-6 select-none">
      {enabled && (
        <InfoBanner
          variant="warning"
          icon="warning"
          text={t('geo-exclusion.warning')}
        />
      )}

      <CardNew>
        <CardHeaderSwitch
          checked={enabled}
          onClick={handleToggle}
          header={t('geo-exclusion.enable')}
        />
        {!enabled && (
          <div className="flex flex-col gap-2 px-4 pt-1 pb-4">
            <p className="text-text-secondary text-sm">
              {t('geo-exclusion.description')}
            </p>
            <p className="text-text-tertiary text-xs">
              {t('geo-exclusion.beta-note')}
            </p>
          </div>
        )}
      </CardNew>

      {enabled && (
        <>
          <Socks5PortCard listenPort={listenPort} onCommitPort={setPort} />

          <ExcludedRegions countries={excludedCountries} />

          <SettingsGroup
            settings={[
              {
                title: t('geo-exclusion.setup-instructions.title'),
                onClick: () => navigate(routes.geoExclusionSetup),
                trailing: (
                  <MsIcon icon="chevron_right" className="text-text-primary" />
                ),
              },
            ]}
          />
        </>
      )}
    </PageAnim>
  );
}

export default GeoExclusion;
