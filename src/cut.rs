use uuid::Uuid;

use super::data_blob::DataBlob;
use super::error::CutError;

#[derive(Debug, Clone)]
pub struct CutSpec {
    pub id: Uuid,
    pub name: String,
    pub x_variable: String,
    pub y_variable: Option<String>,
}

pub trait Cut: std::fmt::Debug {
    fn is_inside(&mut self, blob: &DataBlob);
    fn is_valid(&self) -> bool;
    fn reset(&mut self);
    fn get_spec(&self) -> &CutSpec;
}

#[derive(Debug)]
pub struct Cut1D {
    spec: CutSpec,
    low: f32,
    high: f32,
    is_valid: bool,
}

impl Cut for Cut1D {
    fn is_valid(&self) -> bool {
        self.is_valid
    }

    fn is_inside(&mut self, blob: &DataBlob) {
        self.is_valid = match blob.find(&self.spec.x_variable) {
            Some(x) => *x > self.low && *x < self.high,
            None => false,
        };
    }

    fn reset(&mut self) {
        self.is_valid = false;
    }

    fn get_spec(&self) -> &CutSpec {
        &self.spec
    }
}

impl Cut1D {
    pub fn new(spec: CutSpec, low: f32, high: f32) -> Result<Self, CutError> {
        if low > high || high == low {
            Err(CutError::Invalid1D(low, high))
        } else {
            Ok(Self {
                spec,
                low,
                high,
                is_valid: false,
            })
        }
    }
}

#[derive(Debug)]
pub struct Cut2D {
    spec: CutSpec,
    x_values: Vec<f32>,
    y_values: Vec<f32>,
    is_valid: bool,
}

impl Cut for Cut2D {
    fn is_valid(&self) -> bool {
        self.is_valid
    }

    // Use even odd rule to determine if the point is inside the polygon
    fn is_inside(&mut self, blob: &DataBlob) {
        self.is_valid = false;
        if let Some(y_name) = &self.spec.y_variable {
            let x = match blob.find(&self.spec.x_variable) {
                Some(val) => *val,
                None => return,
            };
            let y = match blob.find(&y_name) {
                Some(val) => *val,
                None => return,
            };

            let mut slope: f32;
            for idx in 0..(self.x_values.len() - 1) {
                if x == self.x_values[idx] && y == self.y_values[idx] {
                    self.is_valid = true;
                    return;
                }

                slope = (x - self.x_values[idx]) * (self.y_values[idx + 1] - self.y_values[idx])
                    - (self.x_values[idx + 1] - self.x_values[idx]) * (y - self.y_values[idx]);

                if slope == 0.0 {
                    self.is_valid = true;
                    return;
                } else if (slope < 0.0) != (self.y_values[idx + 1] < self.y_values[idx]) {
                    self.is_valid = !self.is_valid;
                }
            }
        }
    }

    fn reset(&mut self) {
        self.is_valid = false;
    }

    fn get_spec(&self) -> &CutSpec {
        &self.spec
    }
}

impl Cut2D {
    pub fn new(spec: CutSpec, x_values: Vec<f32>, y_values: Vec<f32>) -> Result<Self, CutError> {
        if spec.y_variable.is_none() || x_values.len() != y_values.len() || x_values.len() < 3 {
            Err(CutError::Invalid2D)
        } else if x_values.first() != x_values.last() || y_values.first() != y_values.last() {
            Err(CutError::Unclosed2D)
        } else {
            Ok(Self {
                spec,
                x_values,
                y_values,
                is_valid: false,
            })
        }
    }
}
