// src/model.rs
use serde::Deserialize;
use std::fs;
use std::sync::LazyLock;

#[derive(Debug, Deserialize)]
pub struct PetModel {
    pub input_cols: Vec<String>,  // must match training order
    pub output_cols: Vec<String>, // ["next_hunger","next_energy","next_happiness","next_level"]
    pub weights: Vec<Vec<f64>>,   // shape: [4][9]
    pub bias: Vec<f64>,           // shape: [4]
}

impl PetModel {
    pub fn predict(&self, x: &[f64]) -> [f64; 4] {
        debug_assert_eq!(self.weights.len(), 4);
        let mut y = [0.0; 4];
        for i in 0..4 {
            let mut s = self.bias[i];
            let row = &self.weights[i];
            debug_assert_eq!(row.len(), x.len(), "input dim mismatch");
            for j in 0..x.len() {
                s += row[j] * x[j];
            }
            y[i] = s;
        }
        y
    }
}

// Load once at startup. Override with PET_MODEL_JSON env if needed.
pub static PET_MODEL: LazyLock<PetModel> = LazyLock::new(|| {
    let path = std::env::var("PET_MODEL_JSON").unwrap_or_else(|_| {
        "/Users/rishabhpoddar/Desktop/trythisapp/bitpet/ml/pet_model.json".to_string()
    });
    let data = fs::read_to_string(&path).expect("failed to read pet_model.json");
    serde_json::from_str(&data).expect("invalid pet_model.json")
});
