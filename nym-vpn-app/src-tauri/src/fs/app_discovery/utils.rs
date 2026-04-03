// lowercase names of excluded apps
const EXCLUDED_APP_NAMES: &[&str] = &["nymvpn"];

pub fn is_excluded(name: &str) -> bool {
    EXCLUDED_APP_NAMES
        .iter()
        .any(|excluded| excluded.eq_ignore_ascii_case(name))
}
