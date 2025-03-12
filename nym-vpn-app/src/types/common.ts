import { Gateway } from './tauri';

export type NodeHop = 'entry' | 'exit';

export type UiTheme = 'Dark' | 'Light';
export type ThemeMode = 'System' | UiTheme;

export type Country = {
  name: string;
  code: string;
};

export type SelectedNode = Country | Gateway;
