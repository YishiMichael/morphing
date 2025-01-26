use super::paint::Paint;

#[derive(Clone)]
pub struct Fill {
    pub options: lyon::tessellation::FillOptions,
    pub paint: Paint,
}
