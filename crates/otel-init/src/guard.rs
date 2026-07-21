use anyhow::Result;
use opentelemetry_sdk::{
    logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};
use tracing::warn;

/// A guard that holds the tracer provider, meter provider, and logger provider and ensures proper cleanup.
///
/// The guard has unique ownership of provider shutdown and cannot be cloned.
///
/// ```compile_fail
/// use otel_init::OtelGuard;
///
/// let guard = OtelGuard::new(None, None, None);
/// let _duplicate = guard.clone();
/// ```
#[derive(Debug)]
pub struct OtelGuard {
    tracer_provider: Option<SdkTracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
    logger_provider: Option<SdkLoggerProvider>,
}

impl OtelGuard {
    /// Create a new guard with the given providers
    pub fn new(
        tracer_provider: Option<SdkTracerProvider>,
        meter_provider: Option<SdkMeterProvider>,
        logger_provider: Option<SdkLoggerProvider>,
    ) -> Self {
        Self {
            tracer_provider,
            meter_provider,
            logger_provider,
        }
    }

    // Set the tracer provider
    pub fn with_tracer_provider(mut self, tracer_provider: SdkTracerProvider) -> Self {
        self.tracer_provider = Some(tracer_provider);
        self
    }

    // Set the meter provider
    pub fn with_meter_provider(mut self, meter_provider: SdkMeterProvider) -> Self {
        self.meter_provider = Some(meter_provider);
        self
    }

    // Set the logger provider
    pub fn with_logger_provider(mut self, logger_provider: SdkLoggerProvider) -> Self {
        self.logger_provider = Some(logger_provider);
        self
    }

    /// Manually shutdown all providers
    ///
    /// This method attempts to shut down all providers, even if some fail.
    pub fn shutdown(mut self) -> Result<()> {
        let mut errors = Vec::new();
        if let Some(tracer_provider) = self.tracer_provider.take()
            && let Err(err) = tracer_provider.shutdown()
        {
            errors.push(err);
        }

        if let Some(meter_provider) = self.meter_provider.take()
            && let Err(err) = meter_provider.shutdown()
        {
            errors.push(err);
        }

        if let Some(logger_provider) = self.logger_provider.take()
            && let Err(err) = logger_provider.shutdown()
        {
            errors.push(err);
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(anyhow::anyhow!(
                "Failed to shutdown some providers: {errors:?}"
            )),
        }
    }
}

// Drop the guard and shutdown the providers
impl Drop for OtelGuard {
    fn drop(&mut self) {
        if let Some(tracer_provider) = self.tracer_provider.take()
            && let Err(err) = tracer_provider.shutdown()
        {
            warn!("Failed to shutdown tracer provider: {err:?}");
        }
        if let Some(meter_provider) = self.meter_provider.take()
            && let Err(err) = meter_provider.shutdown()
        {
            warn!("Failed to shutdown meter provider: {err:?}");
        }
        if let Some(logger_provider) = self.logger_provider.take()
            && let Err(err) = logger_provider.shutdown()
        {
            warn!("Failed to shutdown logger provider: {err:?}");
        }
    }
}
