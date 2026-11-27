use nym_network_defaults::ApiUrl;
use nym_vpn_network_config::{Discovery, Fetcher, NymNetworkDetails};
use serde_json::{Value, json};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

const DISCOVERY: &str = "/api/public/v1/.wellknown/qa/discovery.json";
const ENVS: &str = "/api/public/v1/.wellknown/envs.json";
const DETAILS: &str = "/api/v2/network/details";

fn dead_url() -> String {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    format!("http://127.0.0.1:{port}/api/")
}

async fn serves(route: &str, response: ResponseTemplate) -> MockServer {
    let host = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(route))
        .respond_with(response)
        .mount(&host)
        .await;
    host
}

fn fetcher(urls: &[String]) -> Fetcher {
    let mut seed = Discovery::default_mainnet();
    seed.network_name = "qa".to_owned();
    let api_urls: Vec<ApiUrl> = urls
        .iter()
        .map(|url| ApiUrl {
            url: url.clone(),
            front_hosts: None,
        })
        .collect();
    seed.networking.nym_api_urls = api_urls.clone();
    seed.networking.nym_vpn_api_urls = api_urls;
    Fetcher::new(seed, None).unwrap()
}

async fn fetch_from(urls: &[String]) -> Result<Discovery, String> {
    fetcher(urls)
        .fetch_discovery("qa")
        .await
        .map_err(|err| err.to_string())
}

fn discovery_body(url: &str) -> Value {
    let mut body: Value =
        serde_json::from_str(include_str!("../default/mainnet_discovery.json")).unwrap();
    body["network_name"] = json!("qa");
    body["networking"]["nym_vpn_api_urls"] = json!([{ "url": url }]);
    body
}

// A dead host and a failing one are skipped; the next host that answers is used.
#[tokio::test]
async fn e2e_discovery_falls_through_dead_and_failing_hosts() {
    let failing = serves(DISCOVERY, ResponseTemplate::new(500)).await;
    let good = serves(
        DISCOVERY,
        ResponseTemplate::new(200).set_body_json(discovery_body("https://host-b.test/api/")),
    )
    .await;
    let urls = [
        dead_url(),
        format!("{}/api/", failing.uri()),
        dead_url(),
        format!("{}/api/", good.uri()),
    ];

    let discovery = fetch_from(&urls).await.unwrap();

    assert_eq!(
        discovery.nym_vpn_api_urls()[0].url,
        "https://host-b.test/api/"
    );
}

// When every host fails nothing is returned.
#[tokio::test]
async fn e2e_discovery_fails_when_every_host_fails() {
    let failing = serves(DISCOVERY, ResponseTemplate::new(500)).await;

    let result = fetch_from(&[dead_url(), format!("{}/api/", failing.uri())]).await;

    assert!(result.is_err(), "{result:?}");
}

// The registered networks come first in a refresh, so one failing host must not block it.
#[tokio::test]
async fn e2e_registered_networks_fall_through_a_failing_host() {
    let failing = serves(ENVS, ResponseTemplate::new(500)).await;
    let good = serves(
        ENVS,
        ResponseTemplate::new(200).set_body_json(json!(["mainnet"])),
    )
    .await;
    let urls = [
        dead_url(),
        format!("{}/api/", failing.uri()),
        format!("{}/api/", good.uri()),
    ];

    let networks = fetcher(&urls).fetch_registered_networks().await;

    assert!(networks.is_ok(), "{:?}", networks.err());
}

// The network details come right after discovery, so one failing host must not block them either.
#[tokio::test]
async fn e2e_network_details_fall_through_a_failing_host() {
    let failing = serves(DETAILS, ResponseTemplate::new(500)).await;
    let good = serves(
        DETAILS,
        ResponseTemplate::new(200)
            .set_body_json(json!({ "network": NymNetworkDetails::new_mainnet() })),
    )
    .await;
    let urls = [
        dead_url(),
        format!("{}/api/", failing.uri()),
        format!("{}/api/", good.uri()),
    ];

    let details = fetcher(&urls).fetch_network_details().await;

    assert!(details.is_ok(), "{:?}", details.err());
}
