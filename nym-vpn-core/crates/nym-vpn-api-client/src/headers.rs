use reqwest::RequestBuilder;

#[allow(dead_code)]
pub(crate) const DEVICE_AUTHORIZATION_HEADER: &str = "x-device-authorization";

#[allow(dead_code)]
pub(crate) fn add_device_auth_header(builder: RequestBuilder, jwt: String) -> RequestBuilder {
    builder.header(DEVICE_AUTHORIZATION_HEADER, format!("Bearer {jwt}"))
}

#[allow(dead_code)]
pub(crate) fn add_account_auth_header(builder: RequestBuilder, jwt: String) -> RequestBuilder {
    builder.header(reqwest::header::AUTHORIZATION, format!("Bearer {jwt}"))
}
