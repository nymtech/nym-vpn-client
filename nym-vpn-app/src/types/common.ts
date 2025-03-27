import { Gateway } from './tauri';

export type NodeHop = 'entry' | 'exit';

export type UiTheme = 'dark' | 'light';
export type ThemeMode = 'system' | UiTheme;

export type Country = {
  name: string;
  code: string;
};

export type SelectedNode = Country | Gateway;
