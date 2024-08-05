use reqwest::RequestBuilder;

pub(crate) const DEVICE_AUTHORIZATION_HEADER: &str = "x-device-authorization";

pub(crate) fn add_device_auth_header(builder: RequestBuilder, jwt: String) -> RequestBuilder {
    builder.header(DEVICE_AUTHORIZATION_HEADER, format!("Bearer {jwt}"))
}

pub(crate) fn add_account_auth_header(builder: RequestBuilder, jwt: String) -> RequestBuilder {
    builder.header(reqwest::header::AUTHORIZATION, format!("Bearer {jwt}"))
}