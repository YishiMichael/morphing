use strum::EnumProperty;
use strum::VariantArray;

#[derive(Clone, Copy, Debug, EnumProperty, VariantArray)]
pub enum Color {
    #[strum(props(hex = 0x1C758A))]
    BlueE,
    #[strum(props(hex = 0x29ABCA))]
    BlueD,
    #[strum(props(hex = 0x58C4DD))]
    BlueC,
    #[strum(props(hex = 0x9CDCEB))]
    BlueB,
    #[strum(props(hex = 0xC7E9F1))]
    BlueA,
    #[strum(props(hex = 0x49A88F))]
    TealE,
    #[strum(props(hex = 0x55C1A7))]
    TealD,
    #[strum(props(hex = 0x5CD0B3))]
    TealC,
    #[strum(props(hex = 0x76DDC0))]
    TealB,
    #[strum(props(hex = 0xACEAD7))]
    TealA,
    #[strum(props(hex = 0x699C52))]
    GreenE,
    #[strum(props(hex = 0x77B05D))]
    GreenD,
    #[strum(props(hex = 0x83C167))]
    GreenC,
    #[strum(props(hex = 0xA6CF8C))]
    GreenB,
    #[strum(props(hex = 0xC9E2AE))]
    GreenA,
    #[strum(props(hex = 0xE8C11C))]
    YellowE,
    #[strum(props(hex = 0xF4D345))]
    YellowD,
    #[strum(props(hex = 0xFFFF00))]
    YellowC,
    #[strum(props(hex = 0xFFEA94))]
    YellowB,
    #[strum(props(hex = 0xFFF1B6))]
    YellowA,
    #[strum(props(hex = 0xC78D46))]
    GoldE,
    #[strum(props(hex = 0xE1A158))]
    GoldD,
    #[strum(props(hex = 0xF0AC5F))]
    GoldC,
    #[strum(props(hex = 0xF9B775))]
    GoldB,
    #[strum(props(hex = 0xF7C797))]
    GoldA,
    #[strum(props(hex = 0xCF5044))]
    RedE,
    #[strum(props(hex = 0xE65A4C))]
    RedD,
    #[strum(props(hex = 0xFC6255))]
    RedC,
    #[strum(props(hex = 0xFF8080))]
    RedB,
    #[strum(props(hex = 0xF7A1A3))]
    RedA,
    #[strum(props(hex = 0x94424F))]
    MaroonE,
    #[strum(props(hex = 0xA24D61))]
    MaroonD,
    #[strum(props(hex = 0xC55F73))]
    MaroonC,
    #[strum(props(hex = 0xEC92AB))]
    MaroonB,
    #[strum(props(hex = 0xECABC1))]
    MaroonA,
    #[strum(props(hex = 0x644172))]
    PurpleE,
    #[strum(props(hex = 0x715582))]
    PurpleD,
    #[strum(props(hex = 0x9A72AC))]
    PurpleC,
    #[strum(props(hex = 0xB189C6))]
    PurpleB,
    #[strum(props(hex = 0xCAA3E8))]
    PurpleA,
    #[strum(props(hex = 0x222222))]
    GreyE,
    #[strum(props(hex = 0x444444))]
    GreyD,
    #[strum(props(hex = 0x888888))]
    GreyC,
    #[strum(props(hex = 0xBBBBBB))]
    GreyB,
    #[strum(props(hex = 0xDDDDDD))]
    GreyA,
    #[strum(props(hex = 0xFFFFFF))]
    White,
    #[strum(props(hex = 0x000000))]
    Black,
    #[strum(props(hex = 0x736357))]
    GreyBrown,
    #[strum(props(hex = 0x8B4513))]
    DarkBrown,
    #[strum(props(hex = 0xCD853F))]
    LightBrown,
    #[strum(props(hex = 0xD147BD))]
    Pink,
    #[strum(props(hex = 0xDC75CD))]
    LightPink,
    #[strum(props(hex = 0x00FF00))]
    GreenScreen,
    #[strum(props(hex = 0xFF862F))]
    Orange,
    #[strum(props(hex = 0x58C4DD))]
    Blue,
    #[strum(props(hex = 0x5CD0B3))]
    Teal,
    #[strum(props(hex = 0x83C167))]
    Green,
    #[strum(props(hex = 0xFFFF00))]
    Yellow,
    #[strum(props(hex = 0xF0AC5F))]
    Gold,
    #[strum(props(hex = 0xFC6255))]
    Red,
    #[strum(props(hex = 0xC55F73))]
    Maroon,
    #[strum(props(hex = 0x9A72AC))]
    Purple,
    #[strum(props(hex = 0x888888))]
    Grey,
}

impl From<Color> for palette::Srgb {
    fn from(color: Color) -> Self {
        let hex = color.get_int("hex").unwrap() as u32;
        palette::Srgb::<u8>::from(hex).into()
    }
}

impl From<Color> for palette::Srgba {
    fn from(color: Color) -> Self {
        let hex = color.get_int("hex").unwrap() as u32;
        palette::Srgba::<u8>::from(hex).into()
    }
}
