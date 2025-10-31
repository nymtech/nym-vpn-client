import { StateDispatch } from '../../../../types';
import { kvSet } from '../../../../kvStore';

export const setStreamOptimizedLabelSeen = (dispatch: StateDispatch) => {
  dispatch({ type: 'set-streaming-optimized-label-seen', seen: true });
  kvSet('streaming-optimized-label-seen', true);
};
