pub mod block_all;
pub mod permit_dhcp;

use crate::{
    AllowedEndpoint,
    imp::{Error, wfp},
};

pub fn apply_blocked(
    engine: &wfp::Engine,
    allowed_endpoints: &[AllowedEndpoint],
) -> Result<(), Error> {
    block_all::apply(engine)?;

    Ok(())
}
