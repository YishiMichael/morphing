use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;

use palette::WithAlpha;
use strum::EnumProperty;
use strum::VariantArray;

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub struct Color(palette::Srgba<f32>);

impl Color {
    pub const fn min() -> Self {
        Self(palette::Srgba::new(0.0, 0.0, 0.0, 0.0))
    }

    pub const fn max() -> Self {
        Self(palette::Srgba::new(1.0, 1.0, 1.0, 1.0))
    }

    pub fn with_alpha(self, alpha: f32) -> Self {
        Self(self.0.with_alpha(alpha))
    }
}

impl Deref for Color {
    type Target = palette::Srgba<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Color
where
    T: Into<palette::Srgba>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl FromStr for Color {
    type Err = palette::rgb::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(palette::Srgba::from_str(s)
            .or(palette::Srgb::from_str(s).map(palette::Srgba::from))?
            .into())
    }
}

impl From<Color> for nalgebra::Vector4<f32> {
    fn from(color: Color) -> Self {
        let (r, g, b, a) = color.into_components();
        nalgebra::Vector4::new(r, g, b, a)
    }
}

#[derive(Clone, Copy, Debug, EnumProperty, VariantArray)]
pub enum Palette {
    #[strum(props(hex = "#1C758A"))]
    BlueE,
    #[strum(props(hex = "#29ABCA"))]
    BlueD,
    #[strum(props(hex = "#58C4DD"))]
    BlueC,
    #[strum(props(hex = "#9CDCEB"))]
    BlueB,
    #[strum(props(hex = "#C7E9F1"))]
    BlueA,
    #[strum(props(hex = "#49A88F"))]
    TealE,
    #[strum(props(hex = "#55C1A7"))]
    TealD,
    #[strum(props(hex = "#5CD0B3"))]
    TealC,
    #[strum(props(hex = "#76DDC0"))]
    TealB,
    #[strum(props(hex = "#ACEAD7"))]
    TealA,
    #[strum(props(hex = "#699C52"))]
    GreenE,
    #[strum(props(hex = "#77B05D"))]
    GreenD,
    #[strum(props(hex = "#83C167"))]
    GreenC,
    #[strum(props(hex = "#A6CF8C"))]
    GreenB,
    #[strum(props(hex = "#C9E2AE"))]
    GreenA,
    #[strum(props(hex = "#E8C11C"))]
    YellowE,
    #[strum(props(hex = "#F4D345"))]
    YellowD,
    #[strum(props(hex = "#FFFF00"))]
    YellowC,
    #[strum(props(hex = "#FFEA94"))]
    YellowB,
    #[strum(props(hex = "#FFF1B6"))]
    YellowA,
    #[strum(props(hex = "#C78D46"))]
    GoldE,
    #[strum(props(hex = "#E1A158"))]
    GoldD,
    #[strum(props(hex = "#F0AC5F"))]
    GoldC,
    #[strum(props(hex = "#F9B775"))]
    GoldB,
    #[strum(props(hex = "#F7C797"))]
    GoldA,
    #[strum(props(hex = "#CF5044"))]
    RedE,
    #[strum(props(hex = "#E65A4C"))]
    RedD,
    #[strum(props(hex = "#FC6255"))]
    RedC,
    #[strum(props(hex = "#FF8080"))]
    RedB,
    #[strum(props(hex = "#F7A1A3"))]
    RedA,
    #[strum(props(hex = "#94424F"))]
    MaroonE,
    #[strum(props(hex = "#A24D61"))]
    MaroonD,
    #[strum(props(hex = "#C55F73"))]
    MaroonC,
    #[strum(props(hex = "#EC92AB"))]
    MaroonB,
    #[strum(props(hex = "#ECABC1"))]
    MaroonA,
    #[strum(props(hex = "#644172"))]
    PurpleE,
    #[strum(props(hex = "#715582"))]
    PurpleD,
    #[strum(props(hex = "#9A72AC"))]
    PurpleC,
    #[strum(props(hex = "#B189C6"))]
    PurpleB,
    #[strum(props(hex = "#CAA3E8"))]
    PurpleA,
    #[strum(props(hex = "#222222"))]
    GreyE,
    #[strum(props(hex = "#444444"))]
    GreyD,
    #[strum(props(hex = "#888888"))]
    GreyC,
    #[strum(props(hex = "#BBBBBB"))]
    GreyB,
    #[strum(props(hex = "#DDDDDD"))]
    GreyA,
    #[strum(props(hex = "#FFFFFF"))]
    White,
    #[strum(props(hex = "#000000"))]
    Black,
    #[strum(props(hex = "#736357"))]
    GreyBrown,
    #[strum(props(hex = "#8B4513"))]
    DarkBrown,
    #[strum(props(hex = "#CD853F"))]
    LightBrown,
    #[strum(props(hex = "#D147BD"))]
    Pink,
    #[strum(props(hex = "#DC75CD"))]
    LightPink,
    #[strum(props(hex = "#00FF00"))]
    GreenScreen,
    #[strum(props(hex = "#FF862F"))]
    Orange,
    #[strum(props(hex = "#58C4DD"))]
    Blue,
    #[strum(props(hex = "#5CD0B3"))]
    Teal,
    #[strum(props(hex = "#83C167"))]
    Green,
    #[strum(props(hex = "#FFFF00"))]
    Yellow,
    #[strum(props(hex = "#F0AC5F"))]
    Gold,
    #[strum(props(hex = "#FC6255"))]
    Red,
    #[strum(props(hex = "#C55F73"))]
    Maroon,
    #[strum(props(hex = "#9A72AC"))]
    Purple,
    #[strum(props(hex = "#888888"))]
    Grey,
}

impl From<Palette> for Color {
    fn from(color: Palette) -> Self {
        color.get_str("hex").unwrap().parse().unwrap()
    }
}
