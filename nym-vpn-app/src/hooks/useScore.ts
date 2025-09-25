import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import type { Score } from '../types';

function useScore() {
  const { t } = useTranslation('nodeLocation');

  const getPerformance = useCallback(
    (score: Score) => {
      switch (score) {
        case 'offline':
          return {
            icon: 'signal_cellular_alt_1_bar',
            color: 'text-iron',
            label: t('node-details.perf-score.offline'),
          };
        case 'low':
          return {
            icon: 'signal_cellular_alt_1_bar',
            color: 'text-aphrodisiac',
            label: t('node-details.perf-score.low'),
          };
        case 'medium':
          return {
            icon: 'signal_cellular_alt_2_bar',
            color: 'text-king-nacho',
            label: t('node-details.perf-score.medium'),
          };
        case 'high':
          return {
            icon: 'signal_cellular_alt',
            color: 'text-malachite',
            label: t('node-details.perf-score.high'),
          };
      }
    },
    [t],
  );

  const getLoad = useCallback(
    (score: Score) => {
      switch (score) {
        case 'offline':
          return {
            color: 'text-iron dark:text-bombay',
            label: t('node-details.server-load-score.offline'),
          };
        case 'low':
          return {
            color: 'text-malachite',
            label: t('node-details.server-load-score.low'),
          };
        case 'medium':
          return {
            color: 'text-king-nacho',
            label: t('node-details.server-load-score.medium'),
          };
        case 'high':
          return {
            color: 'text-aphrodisiac',
            label: t('node-details.server-load-score.high'),
          };
      }
    },
    [t],
  );

  return { performance: getPerformance, serverLoad: getLoad };
}

export default useScore;
