use serde::Deserialize;
use warp::{Filter, Rejection, body, reject};

#[derive(Debug)]
pub struct ValidationError(String);
impl reject::Reject for ValidationError {}

#[derive(Deserialize, serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

const MAX_SENSOR_ID_LENGTH: usize = 100;
const MAX_UNIT_LENGTH: usize = 50;
const MAX_NAME_LENGTH: usize = 255;
const MAX_BODY_SIZE: u64 = 1024 * 1024;

pub struct Validator;

impl Validator {
    pub fn sensor_id(id: &str) -> Result<(), String> {
        if id.is_empty() {
            return Err("Sensor ID cannot be empty".to_string());
        }

        if id.len() > MAX_SENSOR_ID_LENGTH {
            return Err(format!(
                "Sensor ID exceeds maximum length of {}",
                MAX_SENSOR_ID_LENGTH
            ));
        }

        if !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(
                "Sensor ID contains invalid characters. Only letters, numbers, '-', and '_' are allowed."
                    .to_string(),
            );
        }

        Ok(())
    }

    pub fn unit(unit: &str) -> Result<(), String> {
        if unit.is_empty() {
            return Err("Unit cannot be empty".to_string());
        }

        if unit.len() > MAX_UNIT_LENGTH {
            return Err(format!(
                "Unit exceeds maximum length of {}",
                MAX_UNIT_LENGTH
            ));
        }

        if !unit
            .chars()
            .all(|c| c.is_alphanumeric() || c == '°' || c == '%' || c == '/' || c == ' ')
        {
            return Err(
                "Unit contains invalid characters. Only letters, numbers, and symbols (°%/ ) are allowed."
                    .to_string(),
            );
        }

        Ok(())
    }

    pub fn value(value: f64) -> Result<(), String> {
        if value.is_nan() || value.is_infinite() {
            return Err("Value must be a valid number".to_string());
        }

        if value.abs() > 1_000_000.0 {
            return Err("Value exceeds maximum allowed range of +/- 1,000,000".to_string());
        }

        Ok(())
    }

    pub fn api_key_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("API key name cannot be empty".to_string());
        }

        if name.len() > MAX_NAME_LENGTH {
            return Err(format!(
                "Name exceeds maximum length of {}",
                MAX_NAME_LENGTH
            ));
        }

        Ok(())
    }

    pub fn body_limit() -> impl Filter<Extract = (), Error = Rejection> + Clone {
        body::content_length_limit(MAX_BODY_SIZE)
    }
}
