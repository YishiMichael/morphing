use super::paint::Paint;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct DashPattern {
    pub dashes: Vec<[f64; 2]>, // [dash_length, space_length]
    pub phase: f64,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Stroke {
    pub dash_pattern: Option<DashPattern>,
    pub options: lyon::tessellation::StrokeOptions,
    pub paint: Paint,
}
