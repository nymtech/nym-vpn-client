import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import type { Score } from '../types';

function useScore() {
  const { t } = useTranslation('node-location');

  const getPerformance = useCallback(
    (score: Score) => {
      switch (score) {
        case 'offline':
          return {
            color: 'text-iron',
            label: t('node-details.perf-score.offline'),
          };
        case 'low':
          return {
            color: 'text-aphrodisiac',
            label: t('node-details.perf-score.low'),
          };
        case 'medium':
          return {
            color: 'text-cheddar dark:text-king-nacho',
            label: t('node-details.perf-score.medium'),
          };
        case 'high':
          return {
            color: 'text-primary',
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
            color: 'text-text-secondary',
            label: t('node-details.server-load-score.offline'),
          };
        case 'low':
          return {
            color: 'text-primary',
            label: t('node-details.server-load-score.low'),
          };
        case 'medium':
          return {
            color: 'text-cheddar dark:text-king-nacho',
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
