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
    <PageAnim className="flex h-full flex-col gap-4 select-none">
      <CardNew>
        <CardHeaderSwitch
          checked={enabled}
          onClick={handleToggle}
          header={t('geo-exclusion.enable')}
        />
      </CardNew>

      {!enabled && (
        <div className="flex flex-col gap-2">
          <CardNew className="p-4">
            <div className="flex flex-col gap-2">
              <p className="text-text-secondary text-sm">
                {t('geo-exclusion.description')}
              </p>
            </div>
          </CardNew>
          <p className="text-text-tertiary text-xs">
            {t('geo-exclusion.beta-note')}
          </p>
        </div>
      )}

      {enabled && (
        <>
          <InfoBanner
            variant="warning"
            icon="warning"
            text={t('geo-exclusion.warning')}
          />
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
