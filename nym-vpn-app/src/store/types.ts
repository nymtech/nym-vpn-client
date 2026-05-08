import { MainSlice } from './slices/createMainSlice';
import { Socks5Slice } from './slices/createSocks5Slice';
import { GatewaysSlice } from './slices/gateways';

export type BoundStore = MainSlice & GatewaysSlice & Socks5Slice;
