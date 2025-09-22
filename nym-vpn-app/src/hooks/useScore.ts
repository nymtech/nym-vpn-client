import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import type { Score } from '../types';

function useScore() {
  const { t } = useTranslation('nodeLocation');

  const getProps = useCallback(
    (score: Score) => {
      switch (score) {
        case 'offline':
          return {
            icon: 'signal_cellular_alt_1_bar',
            color: 'text-iron',
            label: t('node-details.score.offline'),
          };
        case 'low':
          return {
            icon: 'signal_cellular_alt_1_bar',
            color: 'text-aphrodisiac',
            label: t('node-details.score.low'),
          };
        case 'medium':
          return {
            icon: 'signal_cellular_alt_2_bar',
            color: 'text-king-nacho',
            label: t('node-details.score.medium'),
          };
        case 'high':
          return {
            icon: 'signal_cellular_alt',
            color: 'text-malachite',
            label: t('node-details.score.high'),
          };
      }
    },
    [t],
  );

  return { style: getProps };
}

export default useScore;
