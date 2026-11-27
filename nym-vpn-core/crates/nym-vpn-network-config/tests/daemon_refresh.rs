use std::time::Duration;

use nym_network_defaults::ApiUrl;
use nym_offline_monitor::{Connectivity, ConnectivityMonitor};
use nym_vpn_network_config::{DiscoveryRefresher, Network, NetworkCache, NymNetworkDetails};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

struct Online;

#[async_trait::async_trait]
impl ConnectivityMonitor for Online {
    async fn connectivity(&'async_trait self) -> Connectivity {
        Connectivity::PresumeOnline
    }

    async fn next(&'async_trait mut self) -> Option<Connectivity> {
        std::future::pending().await
    }
}

fn urls(list: &[&str]) -> Value {
    json!(
        list.iter()
            .map(|url| json!({ "url": url }))
            .collect::<Vec<_>>()
    )
}

fn vpn_urls(network: &Network) -> Vec<String> {
    network
        .nym_vpn_api_urls()
        .unwrap()
        .into_iter()
        .map(|api_url| api_url.url)
        .collect()
}

// What the daemon has on disk from an earlier run: stale discovery listing `persisted`.
async fn cache_on_disk(dir: &TempDir, persisted: &[&str]) -> NetworkCache {
    NetworkCache::new(dir.path().to_path_buf(), "mainnet", None, None)
        .await
        .unwrap();
    let file = dir.path().join("networks/mainnet/mainnet_discovery.json");
    let mut record: Value = serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    record["updated_at"] = Value::Null;
    record["value"]["networking"]["nym_api_urls"] = urls(persisted);
    record["value"]["networking"]["nym_vpn_api_urls"] = urls(persisted);
    std::fs::write(&file, serde_json::to_vec(&record).unwrap()).unwrap();
    NetworkCache::new(dir.path().to_path_buf(), "mainnet", None, None)
        .await
        .unwrap()
}

fn discovery_body(served: &[&str]) -> ResponseTemplate {
    let mut body: Value =
        serde_json::from_str(include_str!("../default/mainnet_discovery.json")).unwrap();
    body["networking"]["nym_api_urls"] = urls(served);
    body["networking"]["nym_vpn_api_urls"] = urls(served);
    ResponseTemplate::new(200).set_body_json(body)
}

// Serves envs, discovery and nym-api, which always lists only `known`.
async fn serve(server: &MockServer, discovery: ResponseTemplate, known: &str) {
    let mut details = NymNetworkDetails::new_mainnet();
    details.networking.nym_api_urls = vec![ApiUrl {
        url: known.to_owned(),
        front_hosts: None,
    }];
    details.networking.nym_vpn_api_urls = details.networking.nym_api_urls.clone();

    for (route, response) in [
        (
            "/api/public/v1/.wellknown/envs.json",
            ResponseTemplate::new(200).set_body_json(json!(["mainnet"])),
        ),
        (
            "/api/public/v1/.wellknown/mainnet/discovery.json",
            discovery,
        ),
        (
            "/api/v2/network/details",
            ResponseTemplate::new(200).set_body_json(json!({ "network": details })),
        ),
    ] {
        Mock::given(method("GET"))
            .and(path(route))
            .respond_with(response)
            .mount(server)
            .await;
    }
}

fn refresher_events(
    cache: NetworkCache,
) -> (mpsc::UnboundedReceiver<Box<Network>>, CancellationToken) {
    let (_commands_tx, commands_rx) = mpsc::unbounded_channel();
    let (events_tx, events_rx) = mpsc::unbounded_channel();
    let cancel = CancellationToken::new();
    DiscoveryRefresher::spawn(cache, commands_rx, events_tx, Online, cancel.clone());
    (events_rx, cancel)
}

// The hourly refresher pushes a changed list to a running daemon.
#[tokio::test]
async fn e2e_daemon_refresh_emits_the_rotated_urls() {
    let server = MockServer::start().await;
    let host = MockServer::start().await;
    let known = format!("{}/api/", server.uri());
    let other_url = format!("{}/api/", host.uri());
    serve(&server, discovery_body(&[&other_url, &known]), &known).await;
    serve(&host, discovery_body(&[&other_url, &known]), &known).await;
    let dir = tempfile::tempdir().unwrap();
    let cache = cache_on_disk(&dir, &[&known]).await;

    let (mut events, cancel) = refresher_events(cache);
    let network = tokio::time::timeout(Duration::from_secs(5), events.recv())
        .await
        .unwrap()
        .unwrap();
    cancel.cancel();

    assert_eq!(vpn_urls(&network), [other_url.as_str(), known.as_str()]);
}
