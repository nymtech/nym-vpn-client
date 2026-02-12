import { MixnetTrafficConfig } from '../../../../types';
import { NonNullableProps } from '../../../../utils/types';

export type MixnetConfigState = NonNullableProps<MixnetTrafficConfig>;

export type MixnetTrafficConfigAction =
  | {
      type: 'update-field';
      field: keyof MixnetTrafficConfig;
      value: number | boolean;
    }
  | { type: 'update-fields'; state: MixnetConfigState };

export function reducer(
  state: MixnetConfigState,
  action: MixnetTrafficConfigAction,
): MixnetConfigState {
  switch (action.type) {
    case 'update-field':
      return {
        ...state,
        [action.field]: action.value,
      };
    case 'update-fields':
      return action.state;
    default:
      return state;
  }
}
