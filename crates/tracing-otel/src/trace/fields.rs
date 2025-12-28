use http::{HeaderName, Request};

pub const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
pub const REQUEST_ID: HeaderName = HeaderName::from_static("request-id");

/// Extract the http method from the request
pub fn extract_http_method<T>(request: &Request<T>) -> &str {
    request.method().as_str()
}

/// Extract the http route from the request
pub fn extract_http_route<T>(request: &http::Request<T>) -> &str {
    request.uri().path()
}

/// Extract the http version from the request
pub fn extract_http_version<T>(request: &http::Request<T>) -> http::Version {
    request.version()
}

/// Extract the http scheme from the request
pub fn extract_http_scheme<T>(request: &http::Request<T>) -> Option<&str> {
    request.uri().scheme().map(|s| s.as_str())
}

/// Extract the http target from the request
pub fn extract_http_target<T>(request: &http::Request<T>) -> Option<&str> {
    request.uri().path_and_query().map(|s| s.as_str())
}

/// Extract the user agent from the request headers
pub fn extract_user_agent<T>(request: &http::Request<T>) -> Option<&str> {
    extract_field_from_headers(request.headers(), &http::header::USER_AGENT)
}

/// Extract the client ip from the request headers
pub fn extract_host<T>(request: &http::Request<T>) -> Option<&str> {
    extract_field_from_headers(request.headers(), &http::header::HOST)
}

/// Extract the request id from the request headers
pub fn extract_request_id<T>(request: &http::Request<T>) -> &str {
    extract_request_id_from_headers(request.headers()).unwrap_or_default()
}

/// Extract the request id from the request headers
pub fn extract_request_id_from_headers(headers: &http::HeaderMap) -> Option<&str> {
    headers
        .get(X_REQUEST_ID)
        .or_else(|| headers.get(REQUEST_ID))
        .and_then(|value| value.to_str().ok())
}

/// Extract the request id from the request headers
pub fn extract_field_from_headers<'a>(
    headers: &'a http::HeaderMap,
    field: &HeaderName,
) -> Option<&'a str> {
    headers.get(field).and_then(|value| value.to_str().ok())
}

#[cfg(test)]
#[cfg(feature = "trace")]
mod tests {
    use super::*;
    use http::HeaderMap;

    #[test]
    fn test_extract_request_id_with_x_request_id() {
        let mut headers = HeaderMap::new();
        headers.insert(X_REQUEST_ID, "test-id-1".parse().unwrap());
        let request_id = extract_request_id_from_headers(&headers);
        assert_eq!(request_id, Some("test-id-1"));
    }

    #[test]
    fn test_extract_request_id_without_request_id() {
        let request = Request::builder().body(()).unwrap();
        let request_id = extract_request_id(&request);
        assert_eq!(request_id, "");
    }

    #[test]
    fn test_extract_request_id_with_x_request_id_and_request_id() {
        let request = Request::builder()
            .header(X_REQUEST_ID, "test-id-1")
            .header(REQUEST_ID, "test-id-2")
            .body(())
            .unwrap();
        let request_id = extract_request_id(&request);
        assert_eq!(request_id, "test-id-1");
    }

    #[test]
    fn test_get_request_id_without_request_id() {
        let request = Request::builder().body(()).unwrap();
        let request_id = extract_request_id(&request);
        assert_eq!(request_id, "");
    }

    #[test]
    fn test_get_request_id_wit_request_id() {
        let request = Request::builder()
            .header(X_REQUEST_ID, "test-id-1")
            .header(REQUEST_ID, "test-id-2")
            .body(())
            .unwrap();
        let request_id = extract_request_id(&request);
        assert_eq!(request_id, "test-id-1");
    }

    #[test]
    fn test_extract_user_agent() {
        let request = Request::builder()
            .header(http::header::USER_AGENT, "test-user-agent")
            .body(())
            .unwrap();
        let user_agent = extract_user_agent(&request);
        assert_eq!(user_agent, Some("test-user-agent"));
    }

    #[test]
    fn test_extract_host() {
        let request = Request::builder()
            .header(http::header::HOST, "test-host")
            .body(())
            .unwrap();
        let host = extract_host(&request);
        assert_eq!(host, Some("test-host"));
    }
}
