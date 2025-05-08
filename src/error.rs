use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum HistogramError {
    #[error("Histogram has mismatched dimensions")]
    WrongDimensions,
    #[error("Histogram attempted to fill an out of bounds value - min: {0}, max: {1}, val: {2}")]
    OutOfBounds(f32, f32, f32),
    #[error("Invalid axis created: {0}, bins: {1}, min: {2}, max: {2}")]
    BadAxis(String, usize, f32, f32),
}

#[derive(Debug, Error)]
pub enum CutError {
    #[error("Invalid 1D Cut with low: {0} high: {1}")]
    Invalid1D(f32, f32),
    #[error("Invalid 2D Cut with mangled inputs")]
    Invalid2D,
    #[error("Attempted to make 2D Cut with an unclosed polygon")]
    Unclosed2D,
    #[error("Could not find reference histogram {0}")]
    NoReferenceHistogram(Uuid),
}

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("Specter failed to get histogram with ID {0}")]
    InvalidHistogramID(Uuid),
    #[error("Failed to create cut: {0}")]
    CutFailed(#[from] CutError),
}
