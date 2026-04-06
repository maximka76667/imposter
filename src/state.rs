use std::collections::HashMap;

use rand::Rng;
use rand::RngExt;
use serde::Deserialize;

use crate::config::MeasurementType;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SimMode {
    Stable,
    #[default]
    Random,
}

pub struct MeasurementSpec {
    pub id: String,
    pub kind: MeasurementType,
    pub range: Option<[f64; 2]>,
    pub enum_count: usize,
    pub random_step: f64,
}

struct Entry {
    kind: MeasurementType,
    value: f64,
}

pub struct MeasurementState {
    entries: HashMap<String, Entry>,
}

impl MeasurementState {
    pub fn new(specs: &[MeasurementSpec], mode: &SimMode) -> Self {
        let mut rng = rand::rng();
        let entries = specs
            .iter()
            .map(|s| {
                let value = initial_value(s, mode, &mut rng);
                (
                    s.id.clone(),
                    Entry {
                        kind: s.kind.clone(),
                        value,
                    },
                )
            })
            .collect();
        Self { entries }
    }

    pub fn tick(&mut self, specs: &[MeasurementSpec], mode: &SimMode) {
        if let SimMode::Random = mode {
            self.random_walk(specs);
        }
    }

    pub fn get(&self, id: &str) -> Option<(f64, &MeasurementType)> {
        self.entries.get(id).map(|e| (e.value, &e.kind))
    }

    fn random_walk(&mut self, specs: &[MeasurementSpec]) {
        let mut rng = rand::rng();
        for spec in specs {
            let entry = match self.entries.get_mut(&spec.id) {
                Some(e) => e,
                None => continue,
            };
            match spec.kind {
                MeasurementType::Enum => {
                    if spec.enum_count > 0 {
                        entry.value = rng.random_range(0..spec.enum_count) as f64;
                    }
                }
                MeasurementType::Bool => {
                    entry.value = rng.random_range(0..2) as f64;
                }
                _ => {
                    let [min, max] = effective_range(&spec.kind, spec.range);
                    let step = (max - min) * spec.random_step;
                    let delta: f64 = rng.random_range(-step..=step);
                    entry.value = (entry.value + delta).clamp(min, max);
                }
            }
        }
    }
}

fn initial_value(spec: &MeasurementSpec, mode: &SimMode, rng: &mut impl rand::Rng) -> f64 {
    match spec.kind {
        MeasurementType::Enum => {
            if spec.enum_count > 0 {
                rng.random_range(0..spec.enum_count) as f64
            } else {
                0.0
            }
        }
        MeasurementType::Bool => rng.random_range(0..2) as f64,
        _ => {
            let [min, max] = effective_range(&spec.kind, spec.range);
            match mode {
                SimMode::Stable => rng.random_range(min..=max),
                SimMode::Random => (min + max) / 2.0,
            }
        }
    }
}

fn effective_range(kind: &MeasurementType, range: Option<[f64; 2]>) -> [f64; 2] {
    if let Some(r) = range {
        return r;
    }
    match kind {
        MeasurementType::Float32 | MeasurementType::Float64 => [-100.0, 100.0],
        MeasurementType::Uint8 => [0.0, 255.0],
        MeasurementType::Int8 => [-128.0, 127.0],
        MeasurementType::Uint16 => [0.0, 1000.0],
        MeasurementType::Int16 => [-500.0, 500.0],
        MeasurementType::Uint32 | MeasurementType::Int32 => [0.0, 1000.0],
        MeasurementType::Uint64 | MeasurementType::Int64 => [0.0, 1000.0],
        MeasurementType::Bool | MeasurementType::Enum => [0.0, 1.0],
    }
}
