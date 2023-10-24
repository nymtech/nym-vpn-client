use nym_config::defaults::var_names::NYM_API;
use nym_config::OptionalSet;
use nym_validator_client::NymApiClient;
use url::Url;

const DEFAULT_API_URL: &str = "http://127.0.0.1:8000";

pub struct Config {
    api_url: Url,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_url: DEFAULT_API_URL.parse().unwrap(),
        }
    }
}

impl Config {
    pub fn with_custom_api_url(mut self, api_url: Url) -> Self {
        self.api_url = api_url;
        self
    }

    pub fn override_from_env(config: Config) -> Config {
        config.with_optional_env(Config::with_custom_api_url, None, NYM_API)
    }
}

pub struct GatewayClient {
    api_client: NymApiClient,
}

impl GatewayClient {
    pub fn new(config: Config) -> Self {
        let api_client = NymApiClient::new(config.api_url);
        GatewayClient { api_client }
    }

    pub async fn get_host(&self, gateway_identity: &str) -> Result<String, crate::error::Error> {
        self.api_client
            .get_cached_gateways()
            .await?
            .iter()
            .find_map(|gateway_bond| {
                if gateway_bond.identity() == gateway_identity {
                    Some(gateway_bond.gateway().host.clone())
                } else {
                    None
                }
            })
            .ok_or(crate::error::Error::InvalidGatewayID)
    }
}
