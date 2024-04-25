tonic::include_proto!("nym.vpn");

// client implementation only
tonic::include_proto!("grpc.health.v1");

// needed for reflection
pub const VPN_FD_SET: &[u8] = tonic::include_file_descriptor_set!("vpn_descriptor");
