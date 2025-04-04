// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod absolute_security_descriptor;
mod access_rights;
mod acl;
mod acl_entry_list;
mod borrowed_acl;
mod borrowed_explicit_access;
mod borrowed_trustee;
mod explicit_access;
mod relative_security_descriptor;
mod security_attributes;
mod security_info;
mod sid;
mod trustee;

pub use absolute_security_descriptor::AbsoluteSecurityDescriptor;
pub use access_rights::{
    AccessRights, FileAccessRights, GenericAccessRights, StandardAccessRights,
};
pub use acl::Acl;
pub use acl_entry_list::AclEntryList;
pub use borrowed_acl::BorrowedAcl;
pub use borrowed_explicit_access::BorrowedExplicitAccess;
pub use borrowed_trustee::{BorrowedTrustee, TrusteeSpecificInfo};
pub use explicit_access::{AccessMode, AceFlags, ExplicitAccess};
pub use relative_security_descriptor::RelativeSecurityDescriptor;
pub use security_attributes::SecurityAttributes;
pub use security_info::{set_named_security_info, SecurityInfo, SecurityObjectType};
pub use sid::{AccountLookupResult, Sid, WellKnownSid};
pub use trustee::{Trustee, TrusteeForm, TrusteeType};

// Re-export windows types
pub type Result<T> = windows::core::Result<T>;
pub type Error = windows::core::Error;
