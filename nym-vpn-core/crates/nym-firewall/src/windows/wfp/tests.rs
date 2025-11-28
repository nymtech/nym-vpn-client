use super::*;
use crate::imp::wfp::{
    condition::{
        tests::{
            condition_test_address_and_mask_v6, condition_test_address_v4,
            condition_test_address_v6, condition_test_port, condition_test_protocol,
        }, ConditionBuilder, Location,
        Protocol,
    },
    consts::{
        DHCP_CLIENT_PORT_V6, DHCP_SERVER_PORT_V6, LINK_LOCAL_V4, LINK_LOCAL_V6,
        SITE_LOCAL_DHCP_MULTICAST_V6,
    },
    filter::{FilterBuilder, FilterWeight},
};
use nym_windows::str::wstr;
use windows::{
    core::PWSTR,
    Win32::NetworkManagement::WindowsFilteringPlatform::{
        FWPM_FILTER_FLAG_BOOTTIME, FWPM_LAYER_ALE_AUTH_CONNECT_V6, FWP_ACTION_PERMIT, FWP_UINT32,
    },
};

#[test]
fn filter() {
    let name = "Permit outbound DHCP requests (IPv6)";
    let description = "This filter is part of a rule that permits DHCP client traffic";

    let filter_builder = FilterBuilder::new()
        .key(&guids::FILTER_BASELINE_PERMIT_DHCP_OUTBOUND_REQUEST_V6)
        .name(name)
        .description(description)
        .provider(&guids::PROVIDER)
        .layer(&FWPM_LAYER_ALE_AUTH_CONNECT_V6)
        .sublayer(&guids::SUBLAYER_BASELINE)
        .boottime()
        .weight_enum(FilterWeight::medium())
        .permit();

    let conditions = ConditionBuilder::new()
        .protocol(Protocol::Udp)
        .address_and_mask(Location::Local, &LINK_LOCAL_V6, 10)
        .port(Location::Local, DHCP_CLIENT_PORT_V6)
        .address(Location::Local, &LINK_LOCAL_V4)
        .address(Location::Remote, &SITE_LOCAL_DHCP_MULTICAST_V6)
        .port(Location::Remote, DHCP_SERVER_PORT_V6)
        .build()
        .unwrap();

    let filter = filter_builder.build(Some(&conditions)).unwrap();
    let wfp_filter = filter.wfp();

    // name
    assert_eq!(
        wfp_filter.displayData.name.as_ptr() as *const _,
        filter_builder.name.as_ptr()
    );
    let wname = wstr(name);
    unsafe {
        assert_eq!(
            PWSTR(wfp_filter.displayData.name.as_ptr() as *mut _)
                .to_string()
                .unwrap(),
            PWSTR(wname.as_ptr() as *mut _).to_string().unwrap()
        );
    }

    // description
    assert_eq!(
        wfp_filter.displayData.description.as_ptr() as *const _,
        filter_builder.description.as_ptr()
    );
    let wdescription = wstr(description);
    unsafe {
        assert_eq!(
            PWSTR(wfp_filter.displayData.description.as_ptr() as *mut _)
                .to_string()
                .unwrap(),
            PWSTR(wdescription.as_ptr() as *mut _).to_string().unwrap()
        );
    }

    // key
    assert_eq!(
        wfp_filter.filterKey,
        guids::FILTER_BASELINE_PERMIT_DHCP_OUTBOUND_REQUEST_V6
    );

    // provider_key
    let provider_key = filter_builder.provider_key.unwrap();
    let wfp_provider_key = wfp_filter.providerKey as *const _;
    assert_eq!(wfp_provider_key, provider_key.as_ref() as *const _);
    assert_eq!(unsafe { *wfp_provider_key }, *provider_key);

    // layer_key
    assert_eq!(wfp_filter.layerKey, FWPM_LAYER_ALE_AUTH_CONNECT_V6);

    // sublayer_key
    assert_eq!(wfp_filter.subLayerKey, guids::SUBLAYER_BASELINE);

    // flags
    assert_eq!(wfp_filter.flags, FWPM_FILTER_FLAG_BOOTTIME);

    // weight
    assert_eq!(wfp_filter.weight.r#type, FWP_UINT32);
    assert_eq!(
        unsafe { wfp_filter.weight.Anonymous.uint32 },
        FilterWeight::medium().value()
    );

    // action
    assert_eq!(wfp_filter.action.r#type, FWP_ACTION_PERMIT);

    assert_eq!(wfp_filter.numFilterConditions, conditions.len());
    assert_eq!(wfp_filter.filterCondition, conditions.as_ptr() as *mut _);
    condition_test_protocol(&conditions, 0, Protocol::Udp);
    condition_test_address_and_mask_v6(&conditions, 1, Location::Local);
    condition_test_port(&conditions, 2, Location::Local, DHCP_CLIENT_PORT_V6);
    condition_test_address_v4(&conditions, 3, Location::Local);
    condition_test_address_v6(&conditions, 4, Location::Remote);
    condition_test_port(&conditions, 5, Location::Remote, DHCP_SERVER_PORT_V6);
}
