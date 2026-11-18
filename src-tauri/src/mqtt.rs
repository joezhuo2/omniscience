use std::sync::Arc;

use crate::settings::IntegrationSettings;
use crate::telemetry::Telemetry;

pub fn sync(_telemetry: &Arc<Telemetry>, _settings: &IntegrationSettings) {}
