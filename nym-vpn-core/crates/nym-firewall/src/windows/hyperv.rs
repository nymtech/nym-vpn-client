// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::Win32::System::Wmi::WBEM_E_NOT_FOUND;
use wmi::IWbemClassWrapper;

/// Name of the blocking Hyper-V rule.
const BLOCK_OUTBOUND_RULE_ELEMENT_NAME: &str = "Nym VPN outbound block-all rule";

/// Name of the blocking Hyper-V rule.
const BLOCK_INBOUND_RULE_ELEMENT_NAME: &str = "Nym VPN inbound block-all rule";

/// Unique instance ID identifying the outbound blocking Hyper-V rule.
const BLOCK_OUTBOUND_RULE_UUID: &str = "{ed7dee72-7ca3-4728-ad16-e6ee5c465c98}";

/// Unique instance ID identifying the inbound blocking Hyper-V rule.
const BLOCK_INBOUND_RULE_UUID: &str = "{27cf4143-6670-4e33-9d9c-cb6ce685b58e}";

const WMI_NAMESPACE: &str = "root\\standardcimv2";

/// Errors occurring while configuring Hyper-V firewall rules
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to connect to the WMI namespace '{WMI_NAMESPACE}'")]
    ConnectWmi(#[source] wmi::WMIError),
    #[error("failed to obtain Hyper-V rule class")]
    ObtainHyperVClass(#[source] wmi::WMIError),
    #[error("failed to create new instance of Hyper-V rule class")]
    NewRuleInstance(#[source] wmi::WMIError),
    #[error("failed to set rule setting: {0}")]
    SetRuleKey(&'static str, #[source] wmi::WMIError),
    #[error(r#"failed to put the rule "{0}""#)]
    PutInstance(&'static str, #[source] wmi::WMIError),
    #[error(r#"failed to delete rule "{0}""#)]
    DeleteInstance(&'static str, #[source] wmi::WMIError),
}

/// Initialize WMI connection to the ROOT\StandardCIMV2 namespace, which may be used for
/// interacting with Hyper-V rules.
pub fn init_wmi() -> Result<wmi::WMIConnection, Error> {
    let con = wmi::WMIConnection::with_namespace_path(WMI_NAMESPACE).map_err(Error::ConnectWmi)?;

    // Test whether the class is available
    let _ = con
        .get_object("MSFT_NetFirewallHyperVRule")
        .map_err(Error::ObtainHyperVClass)?;

    Ok(con)
}

/// Add a Hyper-V rule that blocks all traffic using WMI (Windows Management Instrumentation).
///
/// Instances of the WMI class `MSFT_NetFirewallHyperVRule` in the namespace "root\standardcimv2"
/// belong to the same firewall ruleset as that visible in PowerShell using the command
/// `Get-NetFirewallHyperVRule`.
///
/// Details about the `MSFT_NetFirewallHyperVRule`, including the meaning of properties, are
/// documented here:
/// https://learn.microsoft.com/en-us/windows/win32/fwp/wmi/wfascimprov/msft-netfirewallhypervrule
///
/// `con` must be a valid WMI connection for the `root\standardcimv2` WMI namespace. Such a connection
/// can be initialized using [`init_wmi`].
pub fn add_blocking_hyperv_firewall_rules(con: &wmi::WMIConnection) -> Result<(), Error> {
    let class = con
        .get_object("MSFT_NetFirewallHyperVRule")
        .map_err(Error::ObtainHyperVClass)?;

    add_blocking_rule(
        con,
        &class,
        BLOCK_OUTBOUND_RULE_ELEMENT_NAME,
        BLOCK_OUTBOUND_RULE_UUID,
        Direction::Outbound,
    )?;
    add_blocking_rule(
        con,
        &class,
        BLOCK_INBOUND_RULE_ELEMENT_NAME,
        BLOCK_INBOUND_RULE_UUID,
        Direction::Inbound,
    )
}

#[repr(i32)]
enum Direction {
    Inbound = 1,
    Outbound = 2,
}

const ELEMENT_NAME_PROPERTY: &str = "ElementName";
const INSTANCE_ID_PROPERTY: &str = "InstanceID";
const ACTION_PROPERTY: &str = "Action";
const ENABLED_PROPERTY: &str = "Enabled";
const DIRECTION_PROPERTY: &str = "Direction";

fn add_blocking_rule(
    con: &wmi::WMIConnection,
    rule_class: &IWbemClassWrapper,
    element_name: &'static str,
    instance_id: &str,
    direction: Direction,
) -> Result<(), Error> {
    let instance = rule_class
        .spawn_instance()
        .map_err(Error::NewRuleInstance)?;

    instance
        .put_property(ELEMENT_NAME_PROPERTY, element_name)
        .map_err(|err| Error::SetRuleKey(ELEMENT_NAME_PROPERTY, err))?;

    instance
        .put_property(INSTANCE_ID_PROPERTY, instance_id)
        .map_err(|err| Error::SetRuleKey(INSTANCE_ID_PROPERTY, err))?;

    // Action: 4 = block
    instance
        .put_property(ACTION_PROPERTY, 4)
        .map_err(|err| Error::SetRuleKey(ACTION_PROPERTY, err))?;

    // Enabled: 1 = enabled
    instance
        .put_property(ENABLED_PROPERTY, 1)
        .map_err(|err| Error::SetRuleKey(ENABLED_PROPERTY, err))?;

    instance
        .put_property(DIRECTION_PROPERTY, direction as i32)
        .map_err(|err| Error::SetRuleKey(DIRECTION_PROPERTY, err))?;

    con.put_instance(&instance)
        .map_err(|error| Error::PutInstance(element_name, error))
}

/// Remove Hyper-V rule previously added by [`add_blocking_hyperv_firewall_rule`]. See the
/// documentation of that function for more details.
///
/// This function succeeds if the rule is not present or has already been removed.
///
/// `con` must be a valid WMI connection for the `root\standardcimv2` WMI namespace. Such a connection
/// can be initialized using [`init_wmi`].
pub fn remove_blocking_hyperv_firewall_rules(con: &wmi::WMIConnection) -> Result<(), Error> {
    remove_blocking_rule(
        con,
        BLOCK_INBOUND_RULE_ELEMENT_NAME,
        BLOCK_INBOUND_RULE_UUID,
    )?;
    remove_blocking_rule(
        con,
        BLOCK_OUTBOUND_RULE_ELEMENT_NAME,
        BLOCK_OUTBOUND_RULE_UUID,
    )
}

fn remove_blocking_rule(
    con: &wmi::WMIConnection,
    element_name: &'static str,
    instance_id: &str,
) -> Result<(), Error> {
    let rule_path = format!(r#"MSFT_NetFirewallHyperVRule.InstanceID="{instance_id}""#);
    con.delete_instance(&rule_path)
        .or_else(|error| map_deletion_err(element_name, error))
}

fn map_deletion_err(element_name: &'static str, err: wmi::WMIError) -> Result<(), Error> {
    if let wmi::WMIError::HResultError { hres } = err
        && hres == WBEM_E_NOT_FOUND.0
    {
        // If the rule doesn't exist, do nothing
        Ok(())
    } else {
        Err(Error::DeleteInstance(element_name, err))
    }
}
