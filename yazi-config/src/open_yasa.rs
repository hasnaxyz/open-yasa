use serde::Deserialize;
use yazi_codegen::{DeserializeOver, DeserializeOver2};
use yazi_shim::cell::SyncCell;

#[derive(Debug, Deserialize, DeserializeOver, DeserializeOver2)]
pub struct OpenYasa {
	pub machines_layer: SyncCell<bool>,
}
