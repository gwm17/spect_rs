use super::error::HistogramError;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub struct AxisSpec {
    pub variable: String,
    pub title: String,
    pub bins: usize,
    pub minimum: f32,
    pub maximum: f32,
}

impl AxisSpec {
    pub fn new(
        variable: &str,
        title: &str,
        bins: usize,
        min: f32,
        max: f32,
    ) -> Result<Self, HistogramError> {
        if min >= max || bins < 1 {
            return Err(HistogramError::BadAxis(title.to_string(), bins, min, max));
        }

        Ok(Self {
            variable: variable.to_string(),
            title: title.to_string(),
            bins,
            minimum: min,
            maximum: max,
        })
    }
    pub fn get_bin_width(&self) -> f32 {
        (self.maximum - self.minimum) / (self.bins as f32)
    }
    pub fn get_bin(&self, value: f32) -> Result<usize, HistogramError> {
        if value < self.minimum || value >= self.maximum {
            return Err(HistogramError::OutOfBounds(
                self.minimum,
                self.maximum,
                value,
            ));
        }
        Ok(((value - self.minimum) / self.get_bin_width()).floor() as usize)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistSpec {
    pub id: Uuid,
    pub name: String,
    pub title: String,
    pub x_axis: AxisSpec,
    pub y_axis: Option<AxisSpec>,
    pub cuts_to_draw: Vec<u64>,
    pub cuts_to_check: Vec<u64>,
}

#[derive(Debug)]
pub struct Histogram {
    pub spec: HistSpec,
    pub data: Vec<u16>,
}

impl Histogram {
    pub fn new(spec: HistSpec) -> Self {
        let data = match &spec.y_axis {
            None => vec![0; spec.x_axis.bins],
            Some(y_axis) => vec![0; spec.x_axis.bins * y_axis.bins],
        };
        Self { spec, data }
    }

    pub fn fill(&mut self, x_value: f32, y_value: Option<f32>) -> Result<usize, HistogramError> {
        let mut bin = self.spec.x_axis.get_bin(x_value)?;
        if let Some(y) = y_value {
            match &self.spec.y_axis {
                None => return Err(HistogramError::WrongDimensions),
                Some(y_axis) => {
                    bin = bin * y_axis.get_bin(y)?;
                    self.data[bin] += 1;
                    return Ok(bin);
                }
            }
        } else if self.spec.y_axis.is_some() {
            return Err(HistogramError::WrongDimensions);
        } else {
            self.data[bin] += 1;
            return Ok(bin);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axis() {
        assert!(AxisSpec::new("var", "var", 600, 0.0, 3600.0).is_ok());
        assert!(AxisSpec::new("var", "var", 0, 0.0, 3600.0).is_err());
        assert!(AxisSpec::new("var", "var", 600, 36000.0, 3600.0).is_err());
        let axis = AxisSpec::new("var", "var", 600, 0.0, 600.0).unwrap();
        let bin = axis.get_bin(0.5).unwrap();
        let bin_width = axis.get_bin_width();
        assert_eq!(bin, 0);
        assert_eq!(bin_width, 1.0);
        assert!(axis.get_bin(-1.0).is_err());
        assert_eq!(axis.variable, "var");
        assert_eq!(axis.title, "var");
    }

    #[test]
    fn test_hist1d() {
        let spec = HistSpec {
            id: Uuid::new_v4(),
            name: String::from("test"),
            title: String::from("test"),
            x_axis: AxisSpec::new("var", "var", 600, 0.0, 600.0).unwrap(),
            y_axis: None,
            cuts_to_draw: vec![],
            cuts_to_check: vec![],
        };

        let mut gram = Histogram::new(spec);
        assert_eq!(gram.data.len(), 600);
        assert!(gram.fill(0.5, None).is_ok());
        assert!(gram.fill(-1.0, None).is_err());
        assert_eq!(gram.spec.name, "test");
        assert_eq!(gram.spec.title, "test");
        assert!(gram.spec.cuts_to_draw.is_empty());
        assert!(gram.spec.cuts_to_check.is_empty());
    }

    #[test]
    fn test_hist2d() {
        let spec = HistSpec {
            id: Uuid::new_v4(),
            name: String::from("test"),
            title: String::from("test"),
            x_axis: AxisSpec::new("var", "var", 600, 0.0, 600.0).unwrap(),
            y_axis: Some(AxisSpec::new("var2", "var2", 600, 0.0, 600.0).unwrap()),
            cuts_to_draw: vec![],
            cuts_to_check: vec![],
        };

        let mut gram = Histogram::new(spec);
        assert_eq!(gram.data.len(), 360_000);
        assert!(gram.fill(0.5, Some(0.5)).is_ok());
        assert!(gram.fill(0.5, None).is_err());
        assert!(gram.fill(-1.0, Some(0.5)).is_err());
        assert!(gram.fill(0.5, Some(-1.0)).is_err());
        assert!(gram.fill(-1.0, Some(-1.0)).is_err());
        assert_eq!(gram.spec.name, "test");
        assert_eq!(gram.spec.title, "test");
        assert!(gram.spec.cuts_to_draw.is_empty());
        assert!(gram.spec.cuts_to_check.is_empty());
    }
}
