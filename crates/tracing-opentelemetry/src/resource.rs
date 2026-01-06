use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;

/// Creates a resource with the given service name and attributes.
///
/// This function builds an OpenTelemetry resource that identifies your service
/// and includes any additional attributes you want to track.
///
/// # Arguments
///
/// * `service_name` - The name of your service
/// * `attributes` - Additional key-value pairs to include in the resource
///
/// # Examples
///
/// ```rust
/// use tracing_opentelemetry_extra::get_resource;
/// use opentelemetry::KeyValue;
///
/// let resource = get_resource(
///     "my-service",
///     &[KeyValue::new("environment", "production")],
/// );
/// ```
pub fn get_resource(service_name: &str, attributes: &[KeyValue]) -> Resource {
    Resource::builder()
        .with_service_name(service_name.to_string())
        .with_attributes(attributes.to_vec())
        .build()
}

#[cfg(test)]
mod tests {
    use super::get_resource;
    use opentelemetry::KeyValue;

    #[test]
    fn test_get_resource() {
        let service_name = "test-service";
        let attributes = vec![
            KeyValue::new("env", "test"),
            KeyValue::new("version", "1.0.0"),
        ];

        let resource = get_resource(service_name, &attributes);

        assert_eq!(
            resource.get(&opentelemetry::Key::new("service.name")),
            Some(opentelemetry::Value::String(service_name.into()))
        );

        assert_eq!(
            resource.get(&opentelemetry::Key::new("env")),
            Some(opentelemetry::Value::String("test".into()))
        );

        assert_eq!(
            resource.get(&opentelemetry::Key::new("version")),
            Some(opentelemetry::Value::String("1.0.0".into()))
        );
    }
}
