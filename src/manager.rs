use super::cut::{Cut, Cut1D, Cut2D, CutSpec};
use super::data_blob::DataBlob;
use super::error::ResourceError;
use super::histogram::{HistSpec, Histogram};
use rustc_hash::FxHashMap;
use uuid::Uuid;

#[derive(Debug)]
pub struct ResourceManager {
    histograms: FxHashMap<Uuid, Histogram>,
    cuts: FxHashMap<Uuid, Box<dyn Cut>>,
    // graphs: Vec<Box<dyn Graph>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            histograms: FxHashMap::default(),
            cuts: FxHashMap::default(),
            // graphs: vec![],
        }
    }

    pub fn add_histogram(&mut self, spec: HistSpec) -> usize {
        let _ = self.histograms.insert(spec.id, Histogram::new(spec));
        self.histograms.len() - 1
    }

    pub fn remove_histogram(&mut self, id: &Uuid) -> Result<(), ResourceError> {
        if !self.histograms.contains_key(id) {
            Err(ResourceError::InvalidHistogramID(*id))
        } else {
            self.histograms.remove_entry(id);
            Ok(())
        }
    }

    pub fn get_histogram_data(&self, id: &Uuid) -> Result<&[u16], ResourceError> {
        match self.histograms.get(id) {
            Some(gram) => Ok(&gram.data),
            None => Err(ResourceError::InvalidHistogramID(*id)),
        }
    }

    pub fn get_histogram_spec(&self, id: &Uuid) -> Result<&HistSpec, ResourceError> {
        match self.histograms.get(id) {
            Some(gram) => Ok(&gram.spec),
            None => Err(ResourceError::InvalidHistogramID(*id)),
        }
    }

    pub fn add_cut_1d(
        &mut self,
        spec: CutSpec,
        low_value: f32,
        high_value: f32,
        histogram_id: &Uuid,
    ) -> Result<(), ResourceError> {
        if let Some(gram) = self.histograms.get_mut(histogram_id) {
            gram.spec.cuts_to_draw.push(spec.id);
        } else {
            return Err(ResourceError::CutFailed(
                super::error::CutError::NoReferenceHistogram(*histogram_id),
            ));
        }
        let _ = self
            .cuts
            .insert(spec.id, Box::new(Cut1D::new(spec, low_value, high_value)?));
        Ok(())
    }

    pub fn add_cut_2d(
        &mut self,
        spec: CutSpec,
        x_values: Vec<f32>,
        y_values: Vec<f32>,
        histogram_id: &Uuid,
    ) -> Result<(), ResourceError> {
        if let Some(gram) = self.histograms.get_mut(histogram_id) {
            gram.spec.cuts_to_draw.push(spec.id);
        } else {
            return Err(ResourceError::CutFailed(
                super::error::CutError::NoReferenceHistogram(*histogram_id),
            ));
        }
        let _ = self
            .cuts
            .insert(spec.id, Box::new(Cut2D::new(spec, x_values, y_values)?));
        Ok(())
    }

    pub fn update(&mut self, data: DataBlob) -> Result<(), ResourceError> {
        for cut in self.cuts.values_mut() {
            cut.is_inside(&data);
        }

        let mut passed_cuts: bool;
        for gram in self.histograms.values_mut() {
            passed_cuts = true;
            for cut_id in gram.spec.cuts_to_check.iter() {
                if let Some(cut) = self.cuts.get(cut_id) {
                    if !cut.is_valid() {
                        passed_cuts = false;
                        break;
                    }
                }
            }
            if !passed_cuts {
                continue;
            }

            let x_val = match data.find(&gram.spec.x_axis.variable) {
                Some(value) => value,
                None => continue,
            };
            if let Some(y_axis) = &gram.spec.y_axis {
                let y_val = match data.find(&y_axis.variable) {
                    Some(value) => value,
                    None => continue,
                };
                match gram.fill(*x_val, Some(*y_val)) {
                    Ok(bin) => println!("Filled bin : {bin}"),
                    Err(e) => println!("Out of bounds: {e}"),
                }
            } else {
                match gram.fill(*x_val, None) {
                    Ok(bin) => println!("Filled bin: {bin}"),
                    Err(e) => println!("Out of bounds: {e}"),
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::histogram::AxisSpec;

    #[test]
    fn test_managed_histogram() {
        let mut manager = ResourceManager::new();
        let spec1 = HistSpec {
            id: Uuid::new_v4(),
            name: String::from("test1"),
            title: String::from("test1"),
            x_axis: AxisSpec::new("var", "var", 600, 0.0, 600.0).unwrap(),
            y_axis: None,
            cuts_to_draw: vec![],
            cuts_to_check: vec![],
        };
        let spec2 = HistSpec {
            id: Uuid::new_v4(),
            name: String::from("test2"),
            title: String::from("test2"),
            x_axis: AxisSpec::new("var", "var", 600, 0.0, 600.0).unwrap(),
            y_axis: Some(AxisSpec::new("var2", "var2", 600, 0.0, 600.0).unwrap()),
            cuts_to_draw: vec![],
            cuts_to_check: vec![],
        };

        manager.add_histogram(spec1.clone());
        manager.add_histogram(spec2.clone());

        let spec_test = manager.get_histogram_spec(&spec1.id);
        assert!(spec_test.is_ok());
        assert!(manager.get_histogram_spec(&Uuid::new_v4()).is_err());
        assert_eq!(*spec_test.unwrap(), spec1);

        assert!(manager.remove_histogram(&spec1.id).is_ok());
        let spec_test = manager.get_histogram_spec(&spec2.id);
        assert!(spec_test.is_ok());
        assert!(manager.get_histogram_spec(&spec1.id).is_err());
        assert_eq!(*spec_test.unwrap(), spec2);

        manager.remove_histogram(&spec2.id).unwrap();
        assert_eq!(manager.histograms.len(), 0);
    }
}
