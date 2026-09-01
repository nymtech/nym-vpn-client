import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { onNonFatalError } from '../errors';
import { useToast } from '../hooks';

// Bridges unhandled errors that did *not* take the UI down into a toast. The
// global handler stays free of React so it can run before anything mounts;
// this component is the part that needs the provider tree, so it only ever
// sees errors raised while the app is healthy.
function NonFatalErrorToasts() {
  const { t } = useTranslation('errors');
  const { add } = useToast();

  useEffect(() => {
    return onNonFatalError(() => {
      add({
        title: t('unknown', { defaultValue: 'Unknown error' }),
        type: 'error',
      });
    });
  }, [add, t]);

  return null;
}

export default NonFatalErrorToasts;
