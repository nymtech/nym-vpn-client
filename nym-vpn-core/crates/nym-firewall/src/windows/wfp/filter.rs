use crate::imp::{
    Error,
    wfp::{self, condition::Conditions},
};
use nym_windows::{error::win32_error, str::wstr};
use std::fmt;
use windows::{
    Win32::{
        Foundation::STATUS_SUCCESS,
        NetworkManagement::WindowsFilteringPlatform::{
            FWP_ACTION_BLOCK, FWP_ACTION_PERMIT, FWP_UINT32, FWP_VALUE0, FWP_VALUE0_0,
            FWPM_ACTION0, FWPM_FILTER_FLAG_BOOTTIME, FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT,
            FWPM_FILTER_FLAG_DISABLED, FWPM_FILTER_FLAG_PERSISTENT, FWPM_FILTER_FLAGS,
            FWPM_FILTER0, FwpmFilterAdd0,
        },
    },
    core::{GUID, PWSTR},
};

///
/// WFP filter builder, using the builder pattern.
///
/// Note: care is taken to hold any filter data, that is passed to WFP via a pointer,
/// in a Box or Vec so the address remains valid until the filter is built.
///
#[derive(Default, Debug)]
pub struct FilterBuilder {
    pub validation: FilterValidation,
    pub key: Option<GUID>,
    pub name: Vec<u16>,
    pub description: Vec<u16>,
    pub provider_key: Option<Box<GUID>>, // Boxed as it's provided to WFP via pointer.
    pub layer_key: Option<GUID>,
    pub sublayer_key: Option<GUID>,
    pub weight: Option<u32>,
    pub flags: FWPM_FILTER_FLAGS,
    pub action: Option<FilterAction>,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self::new_with_validation(FilterValidation::Extra)
    }

    pub fn new_with_validation(validation: FilterValidation) -> Self {
        Self {
            validation,
            ..Default::default()
        }
    }

    pub fn key(mut self, key: &GUID) -> Self {
        self.key = Some(*key);
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = wstr(name);
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = wstr(description);
        self
    }

    pub fn persistent(mut self) -> Self {
        self.flags |= FWPM_FILTER_FLAG_PERSISTENT;
        self
    }

    pub fn boottime(mut self) -> Self {
        self.flags |= FWPM_FILTER_FLAG_BOOTTIME;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.flags |= FWPM_FILTER_FLAG_DISABLED;
        self
    }

    pub fn definitive(mut self) -> Self {
        self.flags |= FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT;
        self
    }

    pub fn provider(mut self, provider_key: &GUID) -> Self {
        self.provider_key = Some(Box::new(*provider_key));
        self
    }

    pub fn layer(mut self, layer_key: &GUID) -> Self {
        self.layer_key = Some(*layer_key);
        self
    }

    pub fn sublayer(mut self, sublayer_key: &GUID) -> Self {
        self.sublayer_key = Some(*sublayer_key);
        self
    }

    pub fn weight(mut self, weight: u32) -> Self {
        self.weight = Some(weight);
        self
    }

    pub fn weight_enum(mut self, weight: FilterWeight) -> Self {
        self.weight = Some(weight.value());
        self
    }

    pub fn block(mut self) -> Self {
        self.action = Some(FilterAction::Block);
        self
    }

    pub fn permit(mut self) -> Self {
        self.action = Some(FilterAction::Permit);
        self
    }

    ///
    /// Note: This does not consume self, for two reasons:
    ///
    /// 1: This type holds data referenced (via pointers) from the WFP filter, and that data
    ///    needs to remain valid until the filter is built.
    /// 2: The filter builder is commonly reused, modifying some fields between builds.
    ///
    pub fn build(&self, conditions: Option<&Conditions>) -> Result<Filter, Error> {
        self.validate()?;

        let mut filter = FWPM_FILTER0::default();

        if let Some(key) = self.key {
            filter.filterKey = key;
        }

        // Late binding of pointers as they would become invalid when using the builder pattern.
        if !self.name.is_empty() {
            filter.displayData.name = PWSTR(self.name.as_ptr() as *mut _);
        }

        if !self.description.is_empty() {
            filter.displayData.description = PWSTR(self.description.as_ptr() as *mut _);
        }

        if let Some(ref provider_key) = self.provider_key {
            filter.providerKey = provider_key.as_ref() as *const _ as *mut _;
        }

        if let Some(layer_key) = self.layer_key {
            filter.layerKey = layer_key;
        }

        if let Some(sublayer_key) = self.sublayer_key {
            filter.subLayerKey = sublayer_key;
        }

        filter.flags = self.flags;

        if let Some(weight) = self.weight {
            filter.weight = FWP_VALUE0 {
                r#type: FWP_UINT32,
                Anonymous: FWP_VALUE0_0 { uint32: weight },
            };
        }

        if let Some(action) = &self.action {
            filter.action = match action {
                FilterAction::Block => FWPM_ACTION0 {
                    r#type: FWP_ACTION_BLOCK,
                    Anonymous: Default::default(),
                },
                FilterAction::Permit => FWPM_ACTION0 {
                    r#type: FWP_ACTION_PERMIT,
                    Anonymous: Default::default(),
                },
            }
        }

        if let Some(conditions) = conditions {
            filter.filterCondition = conditions.as_ptr() as *mut _;
            filter.numFilterConditions = conditions.len();
        }

        Ok(Filter(filter))
    }

    fn validate(&self) -> Result<(), Error> {
        match self.validation {
            FilterValidation::Extra => {
                self.validate_critical()?;
                self.validate_extra()?;
            }
            FilterValidation::OnlyCritical => {
                self.validate_critical()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn validate_critical(&self) -> Result<(), Error> {
        if self.name.is_empty() {
            return Err(Error::Filter {
                reason: "Filter name is required".to_string(),
            });
        }
        if self.layer_key.is_none() {
            return Err(Error::Filter {
                reason: "Filter layer is required".to_string(),
            });
        }
        if self.weight.is_none() {
            return Err(Error::Filter {
                reason: "Filter weight is required".to_string(),
            });
        }
        if self.action.is_none() {
            return Err(Error::Filter {
                reason: "Filter action is required".to_string(),
            });
        }
        Ok(())
    }

    fn validate_extra(&self) -> Result<(), Error> {
        if self.key.is_none() {
            return Err(Error::Filter {
                reason: "Filter key is required".to_string(),
            });
        }
        if self.provider_key.is_none() {
            return Err(Error::Filter {
                reason: "Provider key is required".to_string(),
            });
        }
        if self.sublayer_key.is_none() {
            return Err(Error::Filter {
                reason: "Sublayer is required".to_string(),
            });
        }
        Ok(())
    }
}

impl fmt::Display for FilterBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = {
            let mut flag_strings = Vec::new();
            if self.flags.0 & FWPM_FILTER_FLAG_PERSISTENT.0 != 0 {
                flag_strings.push("PERSISTENT");
            }
            if self.flags.0 & FWPM_FILTER_FLAG_BOOTTIME.0 != 0 {
                flag_strings.push("BOOTTIME");
            }
            if self.flags.0 & FWPM_FILTER_FLAG_DISABLED.0 != 0 {
                flag_strings.push("DISABLED");
            }
            if self.flags.0 & FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT.0 != 0 {
                flag_strings.push("CLEAR_ACTION_RIGHT");
            }
            flag_strings.join("|")
        };
        write!(
            f,
            "Filter {{ Key: {}, Name: {}, Description: {}, Provider Key: {}, Layer Key: {}, Sublayer Key: {}, Weight: {:?}, Flags: {}, Action: {:?} }}",
            self.key
                .as_ref()
                .map_or("None".to_string(), |k| format!("{:?}", k)),
            String::from_utf16_lossy(&self.name),
            String::from_utf16_lossy(&self.description),
            self.provider_key
                .as_ref()
                .map_or("None".to_string(), |k| format!("{:?}", k)),
            self.layer_key
                .as_ref()
                .map_or("None".to_string(), |k| format!("{:?}", k)),
            self.sublayer_key
                .as_ref()
                .map_or("None".to_string(), |k| format!("{:?}", k)),
            self.weight,
            flags,
            self.action,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterAction {
    #[default]
    Block,
    Permit,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterValidation {
    #[default]
    Extra, // Ensure optional values are set if this is considered good practice.
    OnlyCritical, // Perform the bare minimum validation.
    Off,          // No validation
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterWeight {
    Class0 = 0,
    Class1,
    Class2,
    Class3,
    Class4,
    Class5,
    Class6,
    Class7,
    Class8,
    Class9,
    Class10,
    Class11,
    Class12,
    Class13,
    Class14,
    Class15,
}

impl FilterWeight {
    pub fn value(self) -> u32 {
        self as u32
    }

    pub fn min() -> Self {
        FilterWeight::Class0
    }

    pub fn max() -> Self {
        FilterWeight::Class15
    }

    pub fn medium() -> Self {
        FilterWeight::Class7
    }
}

#[derive(Clone)]
pub struct Filter(FWPM_FILTER0);

impl Filter {
    pub fn new(wfp: FWPM_FILTER0) -> Self {
        Self(wfp)
    }

    pub fn wfp(&self) -> &FWPM_FILTER0 {
        &self.0
    }

    /// Add a filter and return its runtime identifier.
    /// Consumes self, in order to reduce misuse.
    pub fn add(self, engine: &wfp::Engine) -> Result<u64, Error> {
        let mut id: u64 = 0;
        let status =
            unsafe { FwpmFilterAdd0(engine.handle(), &self.0, None, Some(&mut id as *mut _)) };

        if status != STATUS_SUCCESS.0 as u32 {
            return Err(Error::Filter {
                reason: format!("FwpmFilterAdd0 failed: {}", win32_error(status)),
            });
        }

        Ok(id)
    }
}
