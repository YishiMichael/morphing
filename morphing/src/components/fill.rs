use serde::Deserialize;
use serde::Serialize;

use super::paint::Paint;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Fill {
    pub options: lyon::tessellation::FillOptions,
    pub paint: Paint,
}
