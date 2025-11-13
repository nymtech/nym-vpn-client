///
/// WFP provider builder, using the builder pattern.
///
/// Note: care is taken to hold any provider data, that is passed to WFP via a pointer,
/// in a Box or Vec so the address remains valid until the filter is built.
/// This means that `build()` does not consume `self` as it owns the data being pointed to.
///
#[derive(Default)]
pub struct ProviderBuilder {
    validation: ProviderValidation,
}

impl ProviderBuilder {
    pub fn new(validation: ProviderValidation) -> Self {
        Self { validation }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ProviderValidation {
    #[default]
    Extra, // Ensure optional values are set if this is considered good practice.
    OnlyCritical, // Perform the bare minimum validation.
    Off,          // No validation
}
