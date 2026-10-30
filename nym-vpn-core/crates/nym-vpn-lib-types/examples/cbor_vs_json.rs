use std::hint::black_box;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::{Duration, Instant};

use nym_vpn_lib_types::*;

fn gateway(i: usize) -> Gateway {
    Gateway {
        identity_key: format!("E3mvZTHQCdBvhfr178Swx9g4QG3kkRUun7YnToLMcMbM{i:04}"),
        name: format!("gateway-node-{i}"),
        description: Some("Community operated exit gateway with IPv6".to_owned()),
        location: Some(Location {
            two_letter_iso_country_code: "DE".to_owned(),
            latitude: 50.1109,
            longitude: 8.6821,
            city: "Frankfurt am Main".to_owned(),
            region: "Hesse".to_owned(),
            asn: Some(Asn { asn: "AS24940".to_owned(), name: "Hetzner Online GmbH".to_owned(), kind: AsnKind::Other }),
        }),
        last_probe: Some(Probe {
            last_updated_utc: "2026-10-28T12:34:56Z".to_owned(),
            outcome: ProbeOutcome {
                as_entry: Entry { can_connect: true, can_route: true },
                as_exit: Some(Exit { can_connect: true, can_route_ip_v4: true, can_route_ip_external_v4: true, can_route_ip_v6: true, can_route_ip_external_v6: false }),
                socks5: None,
                lp: None,
            },
        }),
        mixnet_performance: Some(97),
        bridge_params: None,
        performance: Some(Performance {
            last_updated_utc: "2026-10-28T12:34:56Z".to_owned(),
            score: Score::High,
            mixnet_score: Score::Medium,
            load: Score::Low,
            uptime_percentage_last_24_hours: 99.5,
        }),
        exit_ipv4s: vec![Ipv4Addr::new(95, 216, 1, (i % 255) as u8)],
        exit_ipv6s: vec![Ipv6Addr::new(0x2a01, 0x4f8, 0, 0, 0, 0, 0, i as u16)],
        build_version: Some("1.20.4".to_owned()),
        lewes_protocol_details: None,
        node_family_name: Some("nym-family".to_owned()),
    }
}

fn tunnel_state() -> TunnelState {
    let addr = |g: &str| NymAddress {
        nym_address: format!("8gk4Y5kc7SUmVGqsN4uqYvYNtqsbwchpdlkxq4wzW3M.Ft8uKMH2JyT9JcmSGVjYD5Fr5xMz8uwbUu6yKa9nTj5h@{g}"),
        gateway_id: g.to_owned(),
    };
    TunnelState::Connected {
        connection_data: ConnectionData {
            entry_gateway: GatewayLightInfo { id: "2xU4CBE6QiiYt6EyBXSALwxkNvM7gqJfjHXaMkjiFmYW".to_owned(), country_code: Some("DE".to_owned()) },
            exit_gateway: GatewayLightInfo { id: "6qxtg2wKoyKYF1ns4HSn3bHnxEWVHsHkk8Wm3nDGVVwS".to_owned(), country_code: Some("CH".to_owned()) },
            connected_at: time::OffsetDateTime::now_utc(),
            tunnel: TunnelConnectionData::Mixnet(MixnetConnectionData {
                nym_address: addr("2xU4CBE6QiiYt6EyBXSALwxkNvM7gqJfjHXaMkjiFmYW"),
                exit_ipr: addr("6qxtg2wKoyKYF1ns4HSn3bHnxEWVHsHkk8Wm3nDGVVwS"),
                entry_ip: IpAddr::V4(Ipv4Addr::new(95, 216, 1, 2)),
                exit_ip: IpAddr::V4(Ipv4Addr::new(185, 1, 2, 3)),
                ipv4: Ipv4Addr::new(10, 0, 0, 5),
                ipv6: Some(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 5)),
            }),
        },
    }
}

fn time_it(f: impl Fn()) -> Duration {
    for _ in 0..50 { f(); }
    let mut iters = 1u64;
    loop {
        let t = Instant::now();
        for _ in 0..iters { f(); }
        let e = t.elapsed();
        if e > Duration::from_millis(500) { return e / iters as u32; }
        iters *= 2;
    }
}

fn bench<T>(label: &str, v: &T)
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + minicbor::Encode<()> + for<'b> minicbor::Decode<'b, ()>,
{
    let json = serde_json::to_vec(v).unwrap();
    let cbor = minicbor::to_vec(v).unwrap();
    let je = time_it(|| { black_box(serde_json::to_vec(black_box(v)).unwrap()); });
    let ce = time_it(|| { black_box(minicbor::to_vec(black_box(v)).unwrap()); });
    let jd = time_it(|| { black_box(serde_json::from_slice::<T>(black_box(&json)).unwrap()); });
    let cd = time_it(|| { black_box(minicbor::decode::<T>(black_box(&cbor)).unwrap()); });
    println!("| {label} | {} B | {} B | {:.1?} | {:.1?} | {:.1}x | {:.1?} | {:.1?} | {:.1}x |",
        json.len(), cbor.len(),
        je, ce, je.as_secs_f64() / ce.as_secs_f64(),
        jd, cd, jd.as_secs_f64() / cd.as_secs_f64());
}

fn main() {
    println!("| payload | JSON size | CBOR size | JSON enc | CBOR enc | enc speedup | JSON dec | CBOR dec | dec speedup |");
    println!("|---|---|---|---|---|---|---|---|---|");
    bench("TunnelState::Connected", &tunnel_state());
    bench("Gateway x1", &gateway(1));
    let gws: Vec<Gateway> = (0..300).map(gateway).collect();
    bench("Vec<Gateway> x300", &gws);
}
