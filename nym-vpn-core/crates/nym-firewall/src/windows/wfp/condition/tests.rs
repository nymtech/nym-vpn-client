//
// Note: this module is public so that tests in `super` can perform condition tests in order
// to check that pointers, and the values they point to, are valid and haven't been corrupted.
//

use super::*;
use crate::imp::wfp::condition::{
    Direction, Icmp,
    address::{Address, ConditionAddress},
    address_and_mask::{AddressAndMask, ConditionAddressAndMask},
    application::ConditionApplication,
    direction::ConditionDirection,
    icmp::ConditionIcmp,
    interface::{ConditionInterface, Interface},
    loopback::ConditionLoopback,
    port::ConditionPort,
    port_range::ConditionPortRange,
    protocol::ConditionProtocol,
};
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
};
use windows::Win32::NetworkManagement::{
    IpHelper::IF_TYPE_SOFTWARE_LOOPBACK,
    WindowsFilteringPlatform::{
        FWP_BYTE_ARRAY16_TYPE, FWP_BYTE_BLOB_TYPE, FWP_CONDITION_FLAG_IS_LOOPBACK,
        FWP_DIRECTION_INBOUND, FWP_DIRECTION_OUTBOUND, FWP_RANGE_TYPE, FWP_UINT8, FWP_UINT16,
        FWP_UINT32, FWP_UINT64, FWP_V4_ADDR_MASK, FWP_V6_ADDR_MASK, FWPM_CONDITION_ALE_APP_ID,
        FWPM_CONDITION_DIRECTION, FWPM_CONDITION_FLAGS, FWPM_CONDITION_INTERFACE_INDEX,
        FWPM_CONDITION_INTERFACE_TYPE, FWPM_CONDITION_IP_LOCAL_ADDRESS,
        FWPM_CONDITION_IP_LOCAL_ADDRESS_V4, FWPM_CONDITION_IP_LOCAL_ADDRESS_V6,
        FWPM_CONDITION_IP_LOCAL_INTERFACE, FWPM_CONDITION_IP_LOCAL_PORT,
        FWPM_CONDITION_IP_PROTOCOL, FWPM_CONDITION_IP_REMOTE_ADDRESS,
        FWPM_CONDITION_IP_REMOTE_ADDRESS_V4, FWPM_CONDITION_IP_REMOTE_ADDRESS_V6,
        FWPM_CONDITION_IP_REMOTE_PORT,
    },
};

#[test]
fn address_v4() {
    let ip_addr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let conditions = ConditionBuilder::new()
        .address(Location::Local, &ip_addr)
        .build()
        .unwrap();

    condition_test_address_v4(&conditions, 0, Location::Local);
}

pub fn condition_test_address_v4(conditions: &Conditions, index: usize, location: Location) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionAddress>()
        .expect("Expected ConditionAddress");

    let Address::V4(address) = &condition.address else {
        panic!("Expected Address::V4");
    };

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(
        wfp_condition.fieldKey,
        if location == Location::Local {
            FWPM_CONDITION_IP_LOCAL_ADDRESS_V4
        } else {
            FWPM_CONDITION_IP_REMOTE_ADDRESS_V4
        }
    );
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT32);

    let wfp_value = unsafe { wfp_condition.conditionValue.Anonymous.uint32 };
    assert_eq!(*address, wfp_value);
}

#[test]
fn address_v6() {
    let ip_addr = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
    let conditions = ConditionBuilder::new()
        .address(Location::Remote, &ip_addr)
        .build()
        .unwrap();

    condition_test_address_v6(&conditions, 0, Location::Remote);
}

pub fn condition_test_address_v6(conditions: &Conditions, index: usize, location: Location) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionAddress>()
        .expect("Expected ConditionAddress");

    let Address::V6(address) = &condition.address else {
        panic!("Expected Address::V6");
    };

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(
        wfp_condition.fieldKey,
        if location == Location::Local {
            FWPM_CONDITION_IP_LOCAL_ADDRESS_V6
        } else {
            FWPM_CONDITION_IP_REMOTE_ADDRESS_V6
        }
    );
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_BYTE_ARRAY16_TYPE);

    let wfp_ptr = unsafe { wfp_condition.conditionValue.Anonymous.byteArray16 };
    let expected = address.as_ref();
    assert_eq!(wfp_ptr, expected as *const _ as *mut _);

    let wfp = unsafe { &*wfp_ptr };
    assert_eq!(wfp.byteArray16, expected.byteArray16);
}

#[test]
fn address_and_mask_v4() {
    let ip_addr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let conditions = ConditionBuilder::new()
        .address_and_mask(Location::Local, &ip_addr, 24)
        .build()
        .unwrap();

    condition_test_address_and_mask_v4(&conditions, 0, Location::Local);
}

pub fn condition_test_address_and_mask_v4(
    conditions: &Conditions,
    index: usize,
    location: Location,
) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionAddressAndMask>()
        .expect("Expected ConditionAddressAndMask");

    let AddressAndMask::V4(address_and_mask) = &condition.address_and_mask else {
        panic!("Expected AddressAndMask::V4");
    };

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(
        wfp_condition.fieldKey,
        if location == Location::Local {
            FWPM_CONDITION_IP_LOCAL_ADDRESS
        } else {
            FWPM_CONDITION_IP_REMOTE_ADDRESS
        }
    );
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_V4_ADDR_MASK);

    let wfp_ptr = unsafe { wfp_condition.conditionValue.Anonymous.v4AddrMask };
    let expected = address_and_mask.as_ref();
    assert_eq!(wfp_ptr, expected as *const _ as *mut _);
    let wfp = unsafe { &*wfp_ptr };
    assert_eq!(wfp.addr, expected.addr);
    assert_eq!(wfp.mask, expected.mask);
}

#[test]
fn address_and_mask_v6() {
    let ip_addr = IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1));
    let conditions = ConditionBuilder::new()
        .address_and_mask(Location::Remote, &ip_addr, 64)
        .build()
        .unwrap();

    condition_test_address_and_mask_v6(&conditions, 0, Location::Remote);
}

pub fn condition_test_address_and_mask_v6(
    conditions: &Conditions,
    index: usize,
    location: Location,
) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionAddressAndMask>()
        .expect("Expected ConditionAddressAndMask");

    let AddressAndMask::V6(address_and_mask) = &condition.address_and_mask else {
        panic!("Expected AddressAndMask::V6");
    };

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(
        wfp_condition.fieldKey,
        if location == Location::Local {
            FWPM_CONDITION_IP_LOCAL_ADDRESS
        } else {
            FWPM_CONDITION_IP_REMOTE_ADDRESS
        }
    );
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_V6_ADDR_MASK);

    let wfp_ptr = unsafe { wfp_condition.conditionValue.Anonymous.v6AddrMask };
    let expected = address_and_mask.as_ref();
    assert_eq!(wfp_ptr, expected as *const _ as *mut _);

    let wfp = unsafe { &*wfp_ptr };
    assert_eq!(wfp.addr, expected.addr);
    assert_eq!(wfp.prefixLength, expected.prefixLength);
}

#[test]
fn application() {
    // The application must exist
    let file_path = PathBuf::from(r#"C:\Windows\system32\notepad.exe"#);
    let conditions = ConditionBuilder::new()
        .application(&file_path)
        .unwrap()
        .build()
        .unwrap();

    condition_test_application(&conditions, 0);
}

pub fn condition_test_application(conditions: &Conditions, index: usize) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionApplication>()
        .expect("Expected ConditionApplication");

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_ALE_APP_ID);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_BYTE_BLOB_TYPE);

    let wfp_ptr = unsafe { wfp_condition.conditionValue.Anonymous.byteBlob };
    let expected = condition.app_id_blob.as_ref();
    assert_eq!(wfp_ptr, expected as *const _ as *mut _);

    let wfp = unsafe { &*wfp_ptr };
    assert_eq!(wfp.data, expected.data);
    assert_eq!(wfp.size, expected.size);

    let wfp_data = unsafe { std::slice::from_raw_parts(wfp.data, wfp.size as usize) };
    let expected_data =
        unsafe { std::slice::from_raw_parts(expected.data, expected.size as usize) };
    assert_eq!(wfp_data, expected_data);
}

#[test]
fn direction_inbound() {
    let conditions = ConditionBuilder::new()
        .direction(Direction::Inbound)
        .build()
        .unwrap();

    condition_test_direction_inbound(&conditions, 0);
}

pub fn condition_test_direction_inbound(conditions: &Conditions, index: usize) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionDirection>()
        .expect("Expected ConditionDirection");

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_DIRECTION);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT32);

    assert!(matches!(condition.direction, Direction::Inbound));

    let wfp_value = unsafe { wfp_condition.conditionValue.Anonymous.uint32 };
    assert_eq!(wfp_value, FWP_DIRECTION_INBOUND.0 as u32);
}

#[test]
fn direction_outbound() {
    let conditions = ConditionBuilder::new()
        .direction(Direction::Outbound)
        .build()
        .unwrap();

    condition_test_direction_outbound(&conditions, 0);
}

pub fn condition_test_direction_outbound(conditions: &Conditions, index: usize) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionDirection>()
        .expect("Expected ConditionDirection");

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_DIRECTION);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT32);

    assert!(matches!(condition.direction, Direction::Outbound));

    let wfp_value = unsafe { wfp_condition.conditionValue.Anonymous.uint32 };
    assert_eq!(wfp_value, FWP_DIRECTION_OUTBOUND.0 as u32);
}

#[test]
fn icmp_type() {
    let conditions = ConditionBuilder::new()
        .icmp(Icmp::Type, 12345)
        .build()
        .unwrap();

    condition_test_icmp_type(&conditions, 0);
}

pub fn condition_test_icmp_type(conditions: &Conditions, index: usize) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionIcmp>()
        .expect("Expected ConditionIcmp");

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_IP_LOCAL_PORT);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT32);

    let wfp_value = unsafe { wfp_condition.conditionValue.Anonymous.uint32 };
    assert_eq!(wfp_value, condition.value);
}

#[test]
fn icmp_code() {
    let conditions = ConditionBuilder::new()
        .icmp(Icmp::Code, 23456)
        .build()
        .unwrap();

    condition_test_icmp_code(&conditions, 0);
}

pub fn condition_test_icmp_code(conditions: &Conditions, index: usize) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionIcmp>()
        .expect("Expected ConditionIcmp");

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_IP_REMOTE_PORT);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT32);

    let wfp_value = unsafe { wfp_condition.conditionValue.Anonymous.uint32 };
    assert_eq!(wfp_value, condition.value);
}

#[test]
fn interface_index() {
    let conditions = ConditionBuilder::new().interface_index(1).build().unwrap();

    condition_test_interface_index(&conditions, 0);
}

pub fn condition_test_interface_index(conditions: &Conditions, index: usize) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionInterface>()
        .expect("Expected ConditionInterface");

    let Interface::Index(iface_index) = &condition.interface else {
        panic!("Expected Interface::Index");
    };

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_INTERFACE_INDEX);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT32);

    let wfp_value = unsafe { wfp_condition.conditionValue.Anonymous.uint32 };
    assert_eq!(wfp_value, *iface_index);
}

//  Get-NetAdapter | Format-List -Property IfAlias, IfName, IfType
#[test]
#[ignore = "Won't work on all machines"]
fn interface_name() {
    let conditions = ConditionBuilder::new()
        .interface_name("wireless_32770")
        .unwrap()
        .build()
        .unwrap();

    condition_test_interface_name(&conditions, 0);
}

pub fn condition_test_interface_name(conditions: &Conditions, index: usize) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionInterface>()
        .expect("Expected ConditionInterface");

    let Interface::Luid(luid) = &condition.interface else {
        panic!("Expected Interface::Luid");
    };

    let wfp_condition = &conditions.wfp_conditions[index];
    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_IP_LOCAL_INTERFACE);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT64);

    let wfp_ptr = unsafe { wfp_condition.conditionValue.Anonymous.uint64 };
    let expected = luid.as_ref();
    assert_eq!(wfp_ptr, expected as *const _ as *mut _);

    let wfp = unsafe { &*wfp_ptr };
    assert_eq!(*wfp, *luid.as_ref());
}

//  Get-NetAdapter | Format-List -Property IfAlias, IfName, IfType
#[test]
fn interface_alias() {
    let conditions = ConditionBuilder::new()
        .interface_alias("Loopback Pseudo-Interface 1")
        .unwrap()
        .build()
        .unwrap();

    condition_test_interface_alias(&conditions, 0);
}

pub fn condition_test_interface_alias(conditions: &Conditions, index: usize) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionInterface>()
        .expect("Expected ConditionInterface");

    let Interface::Luid(luid) = &condition.interface else {
        panic!("Expected Interface::Luid");
    };

    let wfp_condition = &conditions.wfp_conditions[index];
    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_IP_LOCAL_INTERFACE);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT64);

    let wfp_ptr = unsafe { wfp_condition.conditionValue.Anonymous.uint64 };
    let expected = luid.as_ref();
    assert_eq!(wfp_ptr, expected as *const _ as *mut _);

    let wfp = unsafe { &*wfp_ptr };
    assert_eq!(*wfp, *luid.as_ref());
}

#[test]
fn loopback_interface() {
    let conditions = ConditionBuilder::new()
        .loopback(Loopback::Interface)
        .build()
        .unwrap();

    condition_test_loopback_interface(&conditions, 0);
}

pub fn condition_test_loopback_interface(conditions: &Conditions, index: usize) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionLoopback>()
        .expect("Expected ConditionLoopback");

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_INTERFACE_TYPE);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_FLAGS_ALL_SET);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT32);

    let wfp_value = unsafe { wfp_condition.conditionValue.Anonymous.uint32 };
    assert_eq!(wfp_value, IF_TYPE_SOFTWARE_LOOPBACK);
}

#[test]
fn loopback_traffic() {
    let conditions = ConditionBuilder::new()
        .loopback(Loopback::Traffic)
        .build()
        .unwrap();

    condition_test_loopback_traffic(&conditions, 0);
}

pub fn condition_test_loopback_traffic(conditions: &Conditions, index: usize) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionLoopback>()
        .expect("Expected ConditionLoopback");

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_FLAGS);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_FLAGS_ALL_SET);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT32);

    let wfp_value = unsafe { wfp_condition.conditionValue.Anonymous.uint32 };
    assert_eq!(wfp_value, FWP_CONDITION_FLAG_IS_LOOPBACK);
}

#[test]
fn port() {
    let port: u16 = 12345;
    let conditions = ConditionBuilder::new()
        .port(Location::Local, port)
        .build()
        .unwrap();

    condition_test_port(&conditions, 0, Location::Local, port);
}

pub fn condition_test_port(conditions: &Conditions, index: usize, location: Location, port: u16) {
    let _condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionPort>()
        .expect("Expected ConditionPort");

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(
        wfp_condition.fieldKey,
        if location == Location::Local {
            FWPM_CONDITION_IP_LOCAL_PORT
        } else {
            FWPM_CONDITION_IP_REMOTE_PORT
        }
    );
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT16);

    let wfp_value = unsafe { wfp_condition.conditionValue.Anonymous.uint16 };
    assert_eq!(wfp_value, port);
}

#[test]
fn port_range() {
    let port_start = 123;
    let port_end = 456;
    let conditions = ConditionBuilder::new()
        .port_range(Location::Local, port_start..port_end)
        .build()
        .unwrap();

    condition_test_port_range(&conditions, 0, port_start, port_end);
}

pub fn condition_test_port_range(
    conditions: &Conditions,
    index: usize,
    port_start: u16,
    port_end: u16,
) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionPortRange>()
        .expect("Expected ConditionPortRange");

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_IP_LOCAL_PORT);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_RANGE);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_RANGE_TYPE);

    let wfp_ptr = unsafe { wfp_condition.conditionValue.Anonymous.rangeValue };
    let expected = condition.port_range.as_ref();
    assert_eq!(wfp_ptr, expected as *const _ as *mut _);

    let wfp = unsafe { &*wfp_ptr };
    assert_eq!(wfp.valueLow.r#type, FWP_UINT16);
    assert_eq!(unsafe { wfp.valueLow.Anonymous.uint16 }, port_start);
    assert_eq!(wfp.valueHigh.r#type, FWP_UINT16);
    assert_eq!(
        unsafe { wfp.valueHigh.Anonymous.uint16 },
        port_end - 1 // Rust range is exclusive, WFP range is inclusive
    );
}

#[test]
fn protocol() {
    let conditions = ConditionBuilder::new()
        .protocol(Protocol::Tcp)
        .build()
        .unwrap();

    condition_test_protocol(&conditions, 0, Protocol::Tcp);
}

pub fn condition_test_protocol(conditions: &Conditions, index: usize, protocol: Protocol) {
    let condition = conditions.conditions[index]
        .as_any()
        .downcast_ref::<ConditionProtocol>()
        .expect("Expected ConditionProtocol");

    let wfp_condition = &conditions.wfp_conditions[index];

    assert_eq!(wfp_condition.fieldKey, FWPM_CONDITION_IP_PROTOCOL);
    assert_eq!(wfp_condition.matchType, FWP_MATCH_EQUAL);
    assert_eq!(wfp_condition.conditionValue.r#type, FWP_UINT8);

    let wfp_value = unsafe { wfp_condition.conditionValue.Anonymous.uint8 };
    let ip_proto: IPPROTO = protocol.into();
    assert_eq!(wfp_value, ip_proto.0 as u8);
}
