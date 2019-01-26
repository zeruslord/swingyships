extern crate toml;

use swingyships::objects::{ChaserDef, ChaserProps};

#[derive(Clone, Debug, Deserialize)]
pub struct Chasers {
    pub defs: Vec<ChaserDef>,
    pub props: ChaserProps
}
