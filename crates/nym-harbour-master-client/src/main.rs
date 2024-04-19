#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gateways = nym_harbour_master_client::get_gateways().await?;
    for gateway in gateways.items {
        println!("{:?}", gateway);
    }
    Ok(())
}
