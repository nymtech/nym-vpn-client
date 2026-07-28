import { Fragment } from 'react';
import { useTranslation } from 'react-i18next';
import { Button } from '@base-ui/react';
import { useNavigate } from 'react-router';
import {
  CardDivider,
  CardNew,
  FlagIcon,
  MsIcon,
  countryCode,
} from '../../../../ui';
import { useLang } from '../../../../hooks';
import { routes } from '../../../../router';

type ExcludedRegionsProps = {
  countries: string[];
};

function ExcludedRegions({ countries }: ExcludedRegionsProps) {
  const { t } = useTranslation('settings');
  const navigate = useNavigate();
  const { getCountryName } = useLang();

  return (
    <div className="flex flex-col gap-2">
      <p className="text-brand-primary px-1 text-xs font-medium tracking-wide uppercase select-none">
        {t('geo-exclusion.excluded-regions.title')}
      </p>
      <CardNew>
        {countries.map((code, index) => (
          <Fragment key={code}>
            {index > 0 && <CardDivider />}
            <div className="flex min-h-16 items-center gap-3 px-4 py-3">
              <FlagIcon code={code.toLowerCase() as countryCode} alt={code} />
              <p className="text-text-primary">
                {getCountryName(code) ?? code}
              </p>
            </div>
          </Fragment>
        ))}
        <CardDivider />
        <Button
          onClick={() => navigate(routes.geoExclusionSelectRegion)}
          className="flex min-h-12 w-full items-center gap-2 px-4 py-3 text-left"
        >
          <MsIcon icon="edit" className="text-text-secondary" />
          <span className="text-text-secondary">
            {t('geo-exclusion.excluded-regions.select-region')}
          </span>
        </Button>
      </CardNew>
    </div>
  );
}

export default ExcludedRegions;
