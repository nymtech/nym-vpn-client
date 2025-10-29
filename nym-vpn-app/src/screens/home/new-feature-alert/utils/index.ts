import { StateAction } from '../../../../contexts/index';
import { kvSet } from '../../../../kvStore/index';
import { DbKey, StateDispatch } from '../../../../types';
import { ACTION_TYPE as STREAMING_OPTIMIZED_LABEL_ACTION_TYPE } from '../streaming-optimized-label/constants';

type AllowedActionTypes = Extract<
  StateAction['type'],
  typeof STREAMING_OPTIMIZED_LABEL_ACTION_TYPE
>;

export const setFeatureSeen = (
  dispatch: StateDispatch,
  action: AllowedActionTypes,
  featureKey: DbKey,
) => {
  dispatch({ type: action, seen: true });
  kvSet(featureKey, true);
};
