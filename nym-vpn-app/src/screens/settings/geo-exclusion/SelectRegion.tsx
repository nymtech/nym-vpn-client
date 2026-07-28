import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';
import {
  FlagIcon,
  PageAnim,
  RadioGroup,
  RadioGroupOption,
  countryCode,
} from '../../../ui';
import { useLang } from '../../../hooks';
import { useGeoExclusion } from './utils/useGeoExclusion';
import {
  SupportedExcludedRegion,
  SupportedExcludedRegions,
} from './utils/regions';

function SelectRegion() {
  const { t } = useTranslation('settings');
  const navigate = useNavigate();
  const { getCountryName } = useLang();
  const { excludedCountries, setExcludedCountry } = useGeoExclusion();

  const options = useMemo<RadioGroupOption<SupportedExcludedRegion>[]>(() => {
    return SupportedExcludedRegions.map((code) => ({
      key: code,
      label: getCountryName(code) ?? code,
      icon: <FlagIcon code={code.toLowerCase() as countryCode} alt={code} />,
    }));
  }, [getCountryName]);

  const handleChange = async (region: SupportedExcludedRegion) => {
    await setExcludedCountry(region);
    navigate(-1);
  };

  return (
    <PageAnim
      className="flex h-full flex-col gap-6 py-6"
      data-testid="geo-exclusion-select-region-page"
    >
      <RadioGroup
        defaultValue={excludedCountries[0] as SupportedExcludedRegion}
        options={options}
        onChange={handleChange}
        data-testid="geo-exclusion-region-radio-group"
      />
      <p className="text-text-tertiary px-1 text-xs">
        {t('geo-exclusion.select-region.beta-note')}
      </p>
    </PageAnim>
  );
}

export default SelectRegion;
