import * as _ from 'lodash-es';
import { AppError, GatewayType, GatewaysByCountry } from '../../types';
import { getStateProps } from './util';
import { GatewaysState } from './types';

export type StateAction =
  | {
      type: 'set-gateways';
      payload: { type: GatewayType; gateways: GatewaysByCountry[] };
    }
  | {
      type: 'set-gateways-loading';
      payload: { type: GatewayType; loading: boolean };
    }
  | {
      type: 'set-gateways-error';
      payload: { type: GatewayType; error: AppError | null };
    }
  | {
      type: 'reset-loading-and-error';
      payload: { type: GatewayType };
    };

export function reducer(
  state: GatewaysState,
  action: StateAction,
): GatewaysState {
  const { gateways, loading, error } = getStateProps(action.payload.type);

  switch (action.type) {
    case 'set-gateways': {
      if (!_.isEqual(action.payload.gateways, state[gateways])) {
        return {
          ...state,
          [gateways]: action.payload.gateways,
        };
      }
      return state;
    }
    case 'set-gateways-loading':
      return {
        ...state,
        [loading]: action.payload.loading,
      };
    case 'set-gateways-error':
      return {
        ...state,
        [error]: action.payload.error,
        [loading]: false,
      };
    case 'reset-loading-and-error':
      return {
        ...state,
        [loading]: false,
        [error]: null,
      };
  }
}
