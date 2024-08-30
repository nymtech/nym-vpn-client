use std::{net::Ipv4Addr, str::FromStr, sync::Arc, time::Duration};

use boringtun::noise::{Tunn, TunnResult};
use log::{error, info};
use nym_connection_monitor::packet_helpers::{create_icmpv4_echo_request, wrap_icmp_in_ipv4};
use pnet_packet::Packet;
use tokio::{net::UdpSocket, sync::Mutex};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::{bind_udp_socket_in_range, icmp::icmp_identifier, MAX_PACKET};

async fn connected_sock_pair() -> (Arc<UdpSocket>, Arc<UdpSocket>) {
    let (sock_a, port_a) = bind_udp_socket_in_range("127.0.0.1", 50000, 65000)
        .await
        .unwrap();
    let (sock_b, port_b) = bind_udp_socket_in_range("127.0.0.1", 50000, 65000)
        .await
        .unwrap();

    let addr_a = format!("127.0.0.1:{}", port_a);
    let addr_b = format!("127.0.0.1:{}", port_b);
    sock_a.connect(&addr_b).await.unwrap();
    sock_b.connect(&addr_a).await.unwrap();
    (Arc::new(sock_a), Arc::new(sock_b))
}

pub async fn wireguard_test_peer<'a>(
    network_socket: Arc<UdpSocket>,
    static_private: StaticSecret,
    peer_static_public: PublicKey,
    spoofed_src_address: &str,
) -> anyhow::Result<bool> {
    let peer = Tunn::new(static_private, peer_static_public, None, Some(25), 0, None);
    // peer.set_logger(logger, Verbosity::Trace);

    let peer: Arc<Mutex<Tunn>> = Arc::from(Mutex::new(peer));

    let (iface_socket_ret, iface_socket) = connected_sock_pair().await;
    info!("iface_socket_ret {:?}", iface_socket_ret.local_addr());
    info!("iface_socket {:?}", iface_socket.local_addr());

    // The peer has three threads:
    // 1) listens on the network for encapsulated packets and decapsulates them
    // 2) listens on the iface for raw packets and encapsulates them
    // 3) times maintenance function responsible for state expiration

    // 1) listens on the network for encapsulated packets and decapsulates them
    // let network_socket = network_socket.try_clone().unwrap();
    // let iface_socket = iface_socket.try_clone().unwrap();
    let peer = peer.clone();

    let peer1 = Arc::clone(&peer);
    let iface_socket1 = Arc::clone(&iface_socket);
    let network_socket1 = Arc::clone(&network_socket);
    let wg_handle =
        tokio::spawn(async move { incoming_wg(peer1, iface_socket1, network_socket1).await });

    let peer2 = Arc::clone(&peer);
    let iface_socket2 = Arc::clone(&iface_socket);
    let network_socket2 = Arc::clone(&network_socket);
    tokio::spawn(async move {
        incoming_ping(peer2, iface_socket2, network_socket2).await;
    });

    let peer3 = Arc::clone(&peer);
    let network_socket3 = Arc::clone(&network_socket);
    tokio::spawn(async move {
        wg_maintenance(peer3, network_socket3).await;
    });

    let iface_socket_ret1 = Arc::clone(&iface_socket_ret);

    let addr = spoofed_src_address.to_string();

    tokio::spawn(async move {
        for i in 0..10 {
            write_ipv4_ping(
                Arc::clone(&iface_socket_ret1),
                i as u16,
                Ipv4Addr::from_str(&addr).unwrap(),
                Ipv4Addr::from_str("8.8.8.8").unwrap(),
            )
            .await
            .unwrap();
        }
    });

    // TODO: Listen for ping responses
    // match tokio::time::timeout(
    //     Duration::from_millis(10000),
    //     read_ipv4_ping(Arc::clone(&iface_socket_ret)),
    // )
    // .await
    // {
    //     Ok(_) => {
    //         info!("Received ICMP Echo Reply");
    //     }
    //     Err(_) => {
    //         error!("Did not receive ICMP Echo Reply");
    //     }
    // }

    match tokio::time::timeout(Duration::from_secs(3), wg_handle).await {
        Ok(can_handshake) => {
            info!("Wireguard test peer finished");
            Ok(can_handshake?)
        }
        Err(_addr) => {
            error!("Wireguard test peer timed out");
            Ok(false)
        }
    }
    // iface_socket_ret
}

pub async fn write_ipv4_ping(
    socket: Arc<UdpSocket>,
    sequence_number: u16,
    src_ip: Ipv4Addr,
    dest_ip: Ipv4Addr,
) -> anyhow::Result<()> {
    let icmp_identifier = icmp_identifier();
    let icmp_echo_request = create_icmpv4_echo_request(sequence_number, icmp_identifier)?;
    let ipv4_packet = wrap_icmp_in_ipv4(icmp_echo_request, src_ip, dest_ip)?;

    socket.send(ipv4_packet.packet()).await?;
    tokio::time::sleep(Duration::from_millis(1000)).await;
    Ok(())
}

// Validate a ping reply packet
#[allow(dead_code)]
pub async fn read_ipv4_ping(socket: Arc<UdpSocket>) {
    let mut data = [0u8; MAX_PACKET];
    // let mut packet = Vec::new();
    if let Ok(len) = socket.recv(&mut data).await {
        info!("Received WG ICMP Echo {}", len);
    }
}

async fn wg_maintenance(peer: Arc<Mutex<Tunn>>, network_socket: Arc<UdpSocket>) {
    let mut send_buf = [0u8; MAX_PACKET];
    let mut w_peer = peer.lock().await;
    match w_peer.update_timers(&mut send_buf) {
        TunnResult::WriteToNetwork(packet) => {
            network_socket.send(packet).await.unwrap();
        }
        _ => {
            // no point printing here as maintenence functions spam hard
            // info!("Thread 3 {:?}", x);
        }
    }

    tokio::time::sleep(Duration::from_millis(200)).await;
}

async fn incoming_ping(
    peer: Arc<Mutex<Tunn>>,
    iface_socket: Arc<UdpSocket>,
    network_socket: Arc<UdpSocket>,
) {
    loop {
        let mut recv_buf = [0u8; MAX_PACKET];
        let mut send_buf = [0u8; MAX_PACKET];

        let n = match iface_socket.recv(&mut recv_buf).await {
            Ok(n) => n,
            Err(e) => {
                error!("ERROR RECEIVED iface: {}", e);
                return;
            }
        };

        // info!("Pre - Encapsulate WriteToNetwork");
        // let mut temp_recv: [u8; 37] = [0u8; 37];
        // temp_recv.copy_from_slice(&recv_buf[0..37]);
        // info!("     iface_socket recv: {:?}", temp_recv);
        let mut w_peer = peer.lock().await;
        match w_peer.encapsulate(&recv_buf[..n], &mut send_buf) {
            TunnResult::WriteToNetwork(packet) => {
                // info!("Encapsulate WriteToNetwork");
                // let parsed_packet = Tunn::parse_incoming_packet(packet).unwrap();
                // info!("Outbound -> WriteToNetwork {:?}", parsed_packet);
                network_socket.send(packet).await.unwrap();
            }
            TunnResult::Done => {}
            TunnResult::Err(err) => {
                error!("Error {:?}", err);
            }
            _ => unreachable!(),
        }
    }
}

async fn incoming_wg(
    peer: Arc<Mutex<Tunn>>,
    iface_socket: Arc<UdpSocket>,
    network_socket: Arc<UdpSocket>,
) -> bool {
    // Listen on the network
    loop {
        let mut recv_buf = [0u8; MAX_PACKET];
        let mut send_buf = [0u8; MAX_PACKET];

        let n = match network_socket.recv_from(&mut recv_buf).await {
            Ok((n, socket_address)) => {
                info!("Incoming WG packet from {}, size = {}", socket_address, n);
                n
            }
            Err(e) => {
                error!("ERROR RECEIVED network: {}", e);
                return false;
            }
        };
        // let mut temp_recv: [u8; 150] = [0u8; 150];
        // temp_recv.copy_from_slice(&recv_buf[0..150]);
        // info!("     network_socket recv: {:?}", temp_recv);
        let mut w_peer = peer.lock().await;
        let packet = Tunn::parse_incoming_packet(&recv_buf[..n]).unwrap();
        #[allow(clippy::single_match)]
        match packet {
            boringtun::noise::Packet::HandshakeResponse(_) => return true,
            _ => {}
        }
        // info!(
        //     "Incoming packet {:?} for {:?} ",
        //     packet,
        //     Tunn::dst_address(&recv_buf[..n])
        // );
        info!("{:?}", w_peer.stats());
        info!("{}", w_peer.is_expired());
        match w_peer.decapsulate(None, &recv_buf[..n], &mut send_buf) {
            TunnResult::WriteToNetwork(packet) => {
                // let parsed_packet = Tunn::parse_incoming_packet(packet).unwrap();
                // info!("WriteToNetwork {:?}", parsed_packet);
                // debug_listenning(packet);
                network_socket.send(packet).await.unwrap();
                // Send form queue?
                loop {
                    let mut send_buf = [0u8; MAX_PACKET];
                    match w_peer.decapsulate(None, &[], &mut send_buf) {
                        TunnResult::WriteToNetwork(packet) => {
                            network_socket.send(packet).await.unwrap();
                        }
                        TunnResult::Done => {
                            break;
                        }
                        TunnResult::Err(err) => {
                            error!("Error {:?}", err);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            TunnResult::WriteToTunnelV4(packet, _) => {
                info!("WriteToTunnelV4");
                info!("       {:?}", packet);
                iface_socket.send(packet).await.unwrap();
            }
            TunnResult::WriteToTunnelV6(packet, _) => {
                info!("WriteToTunnelV6");
                iface_socket.send(packet).await.unwrap();
            }
            TunnResult::Done => {
                info!("Done");
            }
            TunnResult::Err(err) => {
                error!("Error {:?}", err);
            }
        }
        info!("Releasing tunnel lock!")
    }
}

// fn test_smol() {
//     let (mut opts, mut free) = utils::create_options();
//     utils::add_tuntap_options(&mut opts, &mut free);
//     utils::add_middleware_options(&mut opts, &mut free);

//     let mut matches = utils::parse_options(&opts, free);
//     let device = utils::parse_tuntap_options(&mut matches);
// }
