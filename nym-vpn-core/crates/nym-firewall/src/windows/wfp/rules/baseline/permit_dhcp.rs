use windows::Win32::NetworkManagement::WindowsFilteringPlatform::{
    FWPM_LAYER_ALE_AUTH_CONNECT_V4, FWPM_LAYER_ALE_AUTH_CONNECT_V6,
    FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4, FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6,
};

use crate::imp::{
    Error,
    wfp::{self, condition::*, consts::*, filter::*, guids},
};

pub fn apply(engine: &wfp::Engine) -> Result<(), Error> {
    //
    // First UDP packet for a unique [remote address, port] tuple is mapped into:
    //
    // outbound: FWPM_LAYER_ALE_AUTH_CONNECT_V{4|6}
    // inbound: FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V{4|6}
    //

    let mut txn = engine.begin_transaction()?;

    apply_v4(engine)?;

    txn.commit()?;

    Ok(())
}

fn apply_v4(engine: &wfp::Engine) -> Result<(), Error> {
    //
    // #1 Permit outbound DHCPv4 requests.
    //

    let mut filter_builder = FilterBuilder::new()
        .key(&guids::FILTER_BASELINE_PERMIT_DHCP_OUTBOUND_REQUEST_V4)
        .name("Permit outbound DHCP requests (IPv4)")
        .description("This filter is part of a rule that permits DHCP client traffic")
        .provider(&guids::PROVIDER)
        .layer(&FWPM_LAYER_ALE_AUTH_CONNECT_V4)
        .sublayer(&guids::SUBLAYER_BASELINE)
        .weight_enum(FilterWeight::medium())
        .permit();

    {
        let conditions = ConditionBuilder::new()
            .protocol(Protocol::Udp)
            .port(Location::Local, DHCP_CLIENT_PORT_V4)
            .address(Location::Remote, &INADDR_ALL_V4)
            .port(Location::Remote, DHCP_SERVER_PORT_V4)
            .build()?;

        tracing::trace!("{}\n{}", filter_builder, conditions);
        let filter = filter_builder.build(Some(&conditions))?;
        let _id = filter.add(engine)?;
    }

    //
    // #2 Permit inbound DHCPv4 responses (reuses filter builder)
    //

    filter_builder = filter_builder
        .key(&guids::FILTER_BASELINE_PERMIT_DHCP_INBOUND_RESPONSE_V4)
        .name("Permit inbound DHCP responses (IPv4)")
        .layer(&FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4);

    {
        let conditions = ConditionBuilder::new()
            .protocol(Protocol::Udp)
            .port(Location::Local, DHCP_CLIENT_PORT_V4)
            .port(Location::Remote, DHCP_SERVER_PORT_V4)
            .build()?;

        tracing::trace!("{}\n{}", filter_builder, conditions);
        let filter = filter_builder.build(Some(&conditions))?;
        let _id = filter.add(engine)?;
    }

    Ok(())
}

fn apply_v6(_engine: &wfp::Engine) -> Result<(), Error> {
    //
    // #1 Permit outbound DHCPv6 requests.
    //

    let mut filter_builder = FilterBuilder::new()
        .key(&guids::FILTER_BASELINE_PERMIT_DHCP_OUTBOUND_REQUEST_V6)
        .name("Permit outbound DHCP requests (IPv6)")
        .description("This filter is part of a rule that permits DHCP client traffic")
        .provider(&guids::PROVIDER)
        .layer(&FWPM_LAYER_ALE_AUTH_CONNECT_V6)
        .sublayer(&guids::SUBLAYER_BASELINE)
        .weight_enum(FilterWeight::medium())
        .permit();

    {
        let conditions = ConditionBuilder::new()
            .protocol(Protocol::Udp)
            .address_and_mask(Location::Local, &LINK_LOCAL_V6, 10)
            .port(Location::Local, DHCP_CLIENT_PORT_V6)
            .address(Location::Remote, &LINK_LOCAL_DHCP_MULTICAST_V6)
            .address(Location::Remote, &SITE_LOCAL_DHCP_MULTICAST_V6)
            .port(Location::Remote, DHCP_SERVER_PORT_V6)
            .build()?;

        tracing::trace!("{}\n{}", filter_builder, conditions);
        let filter = filter_builder.build(Some(&conditions))?;
        let _id = filter.add(_engine)?;
    }

    //
    // #2 Permit inbound DHCPv6 responses (reuses filter builder)
    //

    filter_builder = filter_builder
        .key(&guids::FILTER_BASELINE_PERMIT_DHCP_INBOUND_RESPONSE_V4)
        .name("Permit inbound DHCP responses (IPv6)")
        .layer(&FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

    {
        let conditions = ConditionBuilder::new()
            .protocol(Protocol::Udp)
            .address_and_mask(Location::Local, &LINK_LOCAL_V6, 10)
            .port(Location::Local, DHCP_CLIENT_PORT_V6)
            .address_and_mask(Location::Remote, &LINK_LOCAL_V6, 10)
            .port(Location::Remote, DHCP_SERVER_PORT_V6)
            .build()?;

        tracing::trace!("{}\n{}", filter_builder, conditions);
        let filter = filter_builder.build(Some(&conditions))?;
        let _id = filter.add(_engine)?;
    }

    Ok(())
}
