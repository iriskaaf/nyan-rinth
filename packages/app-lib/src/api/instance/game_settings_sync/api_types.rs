use super::CanonicalValue;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptionEditorDefinition {
	#[serde(rename = "type")]
	pub type_: String,
	pub min: Option<f64>,
	pub max: Option<f64>,
	pub step: Option<f64>,
	pub unit: Option<String>,
	pub choices: Vec<GameOptionEditorChoice>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameOptionEditorChoice {
	pub value: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameOptionMappingKind {
	Direct,
	Legacy,
	Migrated,
	Lossless,
	Lossy,
}

pub(super) fn canonical_values_equal(
	left: &CanonicalValue,
	right: &CanonicalValue,
) -> bool {
	match (left, right) {
		(CanonicalValue::Decimal(left), CanonicalValue::Decimal(right)) => {
			left.parse::<f64>().ok() == right.parse::<f64>().ok()
		}
		_ => left == right,
	}
}
