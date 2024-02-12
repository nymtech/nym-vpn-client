import React from 'react';

export type InputEvent = React.ChangeEvent<HTMLInputElement>;

export type NodeHop = 'entry' | 'exit';

export type UiTheme = 'Dark' | 'Light';
export type ThemeMode = 'System' | UiTheme;

export type Country = {
  name: string;
  code: string;
};

export type NodeLocation = Country | 'Fastest';

export function isCountry(location: NodeLocation): location is Country {
  return (location as Country).code !== undefined;
}
