import { VpnMode } from './app-state';
import { Country, UiTheme } from './common';

export type NodeLocationBackend = 'Fastest' | { Country: Country };

// tauri type, hence the use of snake_case
export interface AppDataFromBackend {
  monitoring: boolean | null;
  autoconnect: boolean | null;
  killswitch: boolean | null;
  entry_location_selector: boolean | null;
  ui_theme: UiTheme | null;
  ui_root_font_size: number | null;
  vpn_mode: VpnMode | null;
  entry_node_location: NodeLocationBackend | null;
  exit_node_location: NodeLocationBackend | null;
}
