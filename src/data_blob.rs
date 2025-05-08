use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct DataBlob {
    map: FxHashMap<String, f32>,
}

impl DataBlob {
    pub fn find(&self, variable: &str) -> Option<&f32> {
        self.map.get(variable)
    }
}
