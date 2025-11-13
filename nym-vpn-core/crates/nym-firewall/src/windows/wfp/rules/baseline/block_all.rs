use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWPM_LAYER_ALE_AUTH_CONNECT_V4, FWPM_LAYER_ALE_AUTH_CONNECT_V6,
    FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4, FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6,
};

use crate::imp::{
    Error,
    wfp::{self, filter::*, guids},
};

pub fn apply(engine: &wfp::Engine) -> Result<(), Error> {
    let mut txn = engine.begin_transaction()?;

    //
    // #1 Block outbound IPv4 connections
    //

    let mut filter_builder = FilterBuilder::new()
        .key(&guids::FILTER_BASELINE_BLOCK_ALL_OUTBOUND_IPV4)
        .name("Block all outbound connections (IPv4)")
        .description("This filter is part of a rule that restricts inbound and outbound traffic")
        .provider(&guids::PROVIDER)
        .layer(&FWPM_LAYER_ALE_AUTH_CONNECT_V4)
        .sublayer(&guids::SUBLAYER_BASELINE)
        .weight_enum(FilterWeight::min())
        .block();

    {
        tracing::trace!("{}", filter_builder);
        let filter = filter_builder.build(None)?;
        let _id = filter.add(engine)?;
    }

    //
    // #2 Block inbound IPv4 connections (reuses filter builder).
    //

    filter_builder = filter_builder
        .key(&guids::FILTER_BASELINE_BLOCK_ALL_INBOUND_IPV4)
        .name("Block all inbound connections (IPv4)")
        .layer(&FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

    {
        tracing::trace!("{}", filter_builder);
        let filter = filter_builder.build(None)?;
        let _id = filter.add(engine)?;
    }

    //
    // #3 Block outbound IPv6 connections (reuses filter builder).
    //

    filter_builder = filter_builder
        .key(&guids::FILTER_BASELINE_BLOCK_ALL_OUTBOUND_IPV6)
        .name("Block all outbound connections (IPv6)")
        .layer(&FWPM_LAYER_ALE_AUTH_CONNECT_V6);

    {
        tracing::trace!("{}", filter_builder);
        let filter = filter_builder.build(None)?;
        let _id = filter.add(engine)?;
    }

    //
    // #4 Block inbound IPv6 connections (reuses filter builder).
    //

    filter_builder = filter_builder
        .key(&guids::FILTER_BASELINE_BLOCK_ALL_INBOUND_IPV6)
        .name("Block all inbound connections (IPv6)")
        .layer(&FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

    {
        tracing::trace!("{}", filter_builder);
        let filter = filter_builder.build(None)?;
        let _id = filter.add(engine)?;
    }

    txn.commit()?;

    Ok(())
}
