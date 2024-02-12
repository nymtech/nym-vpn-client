import { VpnMode } from './app-state';
import { Country } from './common';

export type NodeLocationBackend = 'Fastest' | { Country: Country };
export type UiThemeBackend = 'System' | 'Dark' | 'Light';

// tauri type, hence the use of snake_case
export interface AppDataFromBackend {
  monitoring: boolean | null;
  autoconnect: boolean | null;
  killswitch: boolean | null;
  entry_location_selector: boolean | null;
  ui_theme: UiThemeBackend | null;
  ui_root_font_size: number | null;
  vpn_mode: VpnMode | null;
  entry_node_location: NodeLocationBackend | null;
  exit_node_location: NodeLocationBackend | null;
}
