use serde::{Deserialize, Serialize};

/// The value shared between instances for one Minecraft setting.
///
/// Minecraft versions sometimes store the same setting in different ways. We
/// convert those formats into one of these values before syncing them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum CanonicalValue {
	Bool(bool),
	Integer(i64),
	Decimal(String),
	Enum(String),
	Text(String),
	StringList(Vec<String>),
	KeyBinding(String),
	ExternalRaw(String),
}
