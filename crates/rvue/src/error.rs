//! Error handling and validation

use std::fmt;

/// Application error types
#[derive(Debug)]
pub enum AppError {
    WindowCreationFailed(String),
    RendererInitializationFailed(String),
    ComponentCreationFailed(String),
    LayoutCalculationFailed(String),
    GcError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::WindowCreationFailed(msg) => write!(f, "Window creation failed: {}", msg),
            AppError::RendererInitializationFailed(msg) => write!(f, "Renderer initialization failed: {}", msg),
            AppError::ComponentCreationFailed(msg) => write!(f, "Component creation failed: {}", msg),
            AppError::LayoutCalculationFailed(msg) => write!(f, "Layout calculation failed: {}", msg),
            AppError::GcError(msg) => write!(f, "GC error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// Validation error types for user input
#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidInput(String),
    OutOfRange { value: f64, min: f64, max: f64 },
    InvalidFormat(String),
    RequiredFieldMissing(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ValidationError::OutOfRange { value, min, max } => {
                write!(f, "Value {} is out of range [{}, {}]", value, min, max)
            }
            ValidationError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ValidationError::RequiredFieldMissing(field) => write!(f, "Required field missing: {}", field),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validation result type
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Input validation functions

/// Validate text input
pub fn validate_text_input(value: &str, min_length: Option<usize>, max_length: Option<usize>) -> ValidationResult<()> {
    if let Some(min) = min_length {
        if value.len() < min {
            return Err(ValidationError::InvalidInput(format!(
                "Text must be at least {} characters",
                min
            )));
        }
    }
    
    if let Some(max) = max_length {
        if value.len() > max {
            return Err(ValidationError::InvalidInput(format!(
                "Text must be at most {} characters",
                max
            )));
        }
    }
    
    Ok(())
}

/// Validate number input
pub fn validate_number_input(value: f64, min: Option<f64>, max: Option<f64>) -> ValidationResult<()> {
    if let Some(min_val) = min {
        if value < min_val {
            return Err(ValidationError::OutOfRange {
                value,
                min: min_val,
                max: max.unwrap_or(f64::MAX),
            });
        }
    }
    
    if let Some(max_val) = max {
        if value > max_val {
            return Err(ValidationError::OutOfRange {
                value,
                min: min.unwrap_or(f64::MIN),
                max: max_val,
            });
        }
    }
    
    Ok(())
}

/// Validate email format (basic)
pub fn validate_email(email: &str) -> ValidationResult<()> {
    if !email.contains('@') || !email.contains('.') {
        return Err(ValidationError::InvalidFormat("Invalid email format".to_string()));
    }
    
    Ok(())
}
