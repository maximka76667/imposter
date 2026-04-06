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

#[cfg(test)]
mod tests {
    use super::*;

    fn float_spec(id: &str, range: Option<[f64; 2]>) -> MeasurementSpec {
        MeasurementSpec {
            id: id.to_string(),
            kind: MeasurementType::Float32,
            range,
            enum_count: 0,
            random_step: 0.1,
        }
    }

    fn enum_spec(id: &str, count: usize) -> MeasurementSpec {
        MeasurementSpec {
            id: id.to_string(),
            kind: MeasurementType::Enum,
            range: None,
            enum_count: count,
            random_step: 0.1,
        }
    }

    fn bool_spec(id: &str) -> MeasurementSpec {
        MeasurementSpec {
            id: id.to_string(),
            kind: MeasurementType::Bool,
            range: None,
            enum_count: 0,
            random_step: 0.1,
        }
    }

    #[test]
    fn random_mode_starts_at_midpoint() {
        let specs = vec![float_spec("v", Some([0.0, 100.0]))];
        let state = MeasurementState::new(&specs, &SimMode::Random);
        let (val, _) = state.get("v").unwrap();
        assert_eq!(val, 50.0);
    }

    #[test]
    fn stable_mode_starts_within_range() {
        let specs = vec![float_spec("v", Some([0.0, 100.0]))];
        let state = MeasurementState::new(&specs, &SimMode::Stable);
        let (val, _) = state.get("v").unwrap();
        assert!(val >= 0.0 && val <= 100.0);
    }

    #[test]
    fn random_tick_changes_values() {
        let specs = vec![float_spec("v", Some([0.0, 100.0]))];
        let mut state = MeasurementState::new(&specs, &SimMode::Random);
        let (initial, _) = state.get("v").unwrap();
        let mut changed = false;
        for _ in 0..100 {
            state.tick(&specs, &SimMode::Random);
            let (val, _) = state.get("v").unwrap();
            if (val - initial).abs() > 1e-10 {
                changed = true;
                break;
            }
        }
        assert!(changed, "value never changed after 100 random ticks");
    }

    #[test]
    fn stable_tick_never_changes() {
        let specs = vec![float_spec("v", Some([0.0, 100.0]))];
        let mut state = MeasurementState::new(&specs, &SimMode::Stable);
        let (initial, _) = state.get("v").unwrap();
        for _ in 0..20 {
            state.tick(&specs, &SimMode::Stable);
        }
        let (val, _) = state.get("v").unwrap();
        assert_eq!(val, initial);
    }

    #[test]
    fn random_walk_clamps_to_range() {
        let specs = vec![float_spec("v", Some([10.0, 20.0]))];
        let mut state = MeasurementState::new(&specs, &SimMode::Random);
        for _ in 0..1000 {
            state.tick(&specs, &SimMode::Random);
            let (val, _) = state.get("v").unwrap();
            assert!(val >= 10.0 && val <= 20.0, "value {val} out of range");
        }
    }

    #[test]
    fn bool_tick_is_zero_or_one() {
        let specs = vec![bool_spec("b")];
        let mut state = MeasurementState::new(&specs, &SimMode::Random);
        for _ in 0..50 {
            state.tick(&specs, &SimMode::Random);
            let (val, _) = state.get("b").unwrap();
            assert!(val == 0.0 || val == 1.0, "Bool value {val} is not 0 or 1");
        }
    }

    #[test]
    fn enum_tick_within_count_and_integer() {
        let specs = vec![enum_spec("e", 5)];
        let mut state = MeasurementState::new(&specs, &SimMode::Random);
        for _ in 0..50 {
            state.tick(&specs, &SimMode::Random);
            let (val, _) = state.get("e").unwrap();
            assert!(val >= 0.0 && val < 5.0, "Enum value {val} out of range");
            assert_eq!(val, val.floor(), "Enum value {val} is not an integer");
        }
    }
}

fn effective_range(kind: &MeasurementType, range: Option<[f64; 2]>) -> [f64; 2] {
    if let Some(r) = range {
        return r;
    }
    match kind {
        MeasurementType::Float32 | MeasurementType::Float64 => [-200.0, 1000.0],
        MeasurementType::Uint8 => [0.0, 255.0],
        MeasurementType::Int8 => [-128.0, 127.0],
        MeasurementType::Uint16 => [0.0, 1000.0],
        MeasurementType::Int16 => [-500.0, 500.0],
        MeasurementType::Uint32 | MeasurementType::Int32 => [0.0, 1000.0],
        MeasurementType::Uint64 | MeasurementType::Int64 => [0.0, 1000.0],
        MeasurementType::Bool | MeasurementType::Enum => [0.0, 1.0],
    }
}
