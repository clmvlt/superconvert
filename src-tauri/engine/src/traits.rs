use std::path::Path;

use crate::error::ConversionError;
use crate::types::ConversionOptions;

#[allow(dead_code)]
pub trait Converter: Send + Sync {
    fn supported_input_formats(&self) -> &[&str];
    fn supported_output_formats(&self) -> &[&str];
    fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: &ConversionOptions,
        on_progress: Box<dyn Fn(f32) + Send>,
    ) -> Result<(), ConversionError>;
}
