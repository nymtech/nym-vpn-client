use nym_harbour_master_client::{Client, HabourMasterError, HarbourMasterApiClientExt};

const HARBOUR_MASTER: &str = "https://harbourmaster.nymtech.net";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new_url::<_, HabourMasterError>(HARBOUR_MASTER, None)?;
    let gateways = client.get_gateways().await?;
    for gateway in gateways.items {
        println!("{:?}", gateway);
    }
    Ok(())
}
