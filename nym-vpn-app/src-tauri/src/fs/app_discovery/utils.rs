// lowercase names of excluded apps
const EXCLUDED_APP_NAMES: &[&str] = &["nymvpn"];

pub fn is_excluded(name: &str) -> bool {
    EXCLUDED_APP_NAMES
        .iter()
        .any(|excluded| excluded.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nymvpn_is_excluded_regardless_of_case() {
        assert!(is_excluded("nymvpn"));
        assert!(is_excluded("NymVPN"));
        assert!(is_excluded("NYMVPN"));
        assert!(is_excluded("Nymvpn"));
    }

    #[test]
    fn unrelated_apps_are_not_excluded() {
        assert!(!is_excluded("Firefox"));
        assert!(!is_excluded("Chrome"));
        assert!(!is_excluded("Spotify"));
        assert!(!is_excluded("Signal"));
    }

    #[test]
    fn empty_string_is_not_excluded() {
        assert!(!is_excluded(""));
    }

    #[test]
    fn superstrings_of_excluded_name_are_not_excluded() {
        // eq_ignore_ascii_case is an exact match, not a substring check
        assert!(!is_excluded("mynymvpn"));
        assert!(!is_excluded("nymvpn-old"));
        assert!(!is_excluded("nymvpn2"));
        assert!(!is_excluded(" nymvpn")); // leading whitespace is a different string
    }
}
