use super::paint::Paint;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Fill {
    pub options: lyon::tessellation::FillOptions,
    pub paint: Paint,
}
