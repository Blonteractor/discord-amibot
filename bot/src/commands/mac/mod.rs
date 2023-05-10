use regex::Regex;

pub mod deregister;
pub mod info;
pub mod register;

// https://stackoverflow.com/questions/4260467/what-is-a-regular-expression-for-a-mac-address
pub fn is_valid_mac(addr: impl AsRef<str>) -> bool {
    let mac_regex = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap();
    mac_regex.is_match(addr.as_ref())
}
