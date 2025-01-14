macro_rules! c {
    ($name:ident = $value:tt) => {
        pub const $name: palette::Srgb<u8> = palette::Srgb::new(
            ($value as u32 >> 16 & 0xFF) as u8,
            ($value as u32 >> 8 & 0xFF) as u8,
            ($value as u32 & 0xFF) as u8,
        );
    };
}

c!(BLUE_E = 0x1C758A);
c!(BLUE_D = 0x29ABCA);
c!(BLUE_C = 0x58C4DD);
c!(BLUE_B = 0x9CDCEB);
c!(BLUE_A = 0xC7E9F1);
c!(TEAL_E = 0x49A88F);
c!(TEAL_D = 0x55C1A7);
c!(TEAL_C = 0x5CD0B3);
c!(TEAL_B = 0x76DDC0);
c!(TEAL_A = 0xACEAD7);
c!(GREEN_E = 0x699C52);
c!(GREEN_D = 0x77B05D);
c!(GREEN_C = 0x83C167);
c!(GREEN_B = 0xA6CF8C);
c!(GREEN_A = 0xC9E2AE);
c!(YELLOW_E = 0xE8C11C);
c!(YELLOW_D = 0xF4D345);
c!(YELLOW_C = 0xFFFF00);
c!(YELLOW_B = 0xFFEA94);
c!(YELLOW_A = 0xFFF1B6);
c!(GOLD_E = 0xC78D46);
c!(GOLD_D = 0xE1A158);
c!(GOLD_C = 0xF0AC5F);
c!(GOLD_B = 0xF9B775);
c!(GOLD_A = 0xF7C797);
c!(RED_E = 0xCF5044);
c!(RED_D = 0xE65A4C);
c!(RED_C = 0xFC6255);
c!(RED_B = 0xFF8080);
c!(RED_A = 0xF7A1A3);
c!(MAROON_E = 0x94424F);
c!(MAROON_D = 0xA24D61);
c!(MAROON_C = 0xC55F73);
c!(MAROON_B = 0xEC92AB);
c!(MAROON_A = 0xECABC1);
c!(PURPLE_E = 0x644172);
c!(PURPLE_D = 0x715582);
c!(PURPLE_C = 0x9A72AC);
c!(PURPLE_B = 0xB189C6);
c!(PURPLE_A = 0xCAA3E8);
c!(GREY_E = 0x222222);
c!(GREY_D = 0x444444);
c!(GREY_C = 0x888888);
c!(GREY_B = 0xBBBBBB);
c!(GREY_A = 0xDDDDDD);
c!(WHITE = 0xFFFFFF);
c!(BLACK = 0x000000);
c!(GREY_BROWN = 0x736357);
c!(DARK_BROWN = 0x8B4513);
c!(LIGHT_BROWN = 0xCD853F);
c!(PINK = 0xD147BD);
c!(LIGHT_PINK = 0xDC75CD);
c!(GREEN_SCREEN = 0x00FF00);
c!(ORANGE = 0xFF862F);

c!(BLUE = 0x58C4DD);
c!(TEAL = 0x5CD0B3);
c!(GREEN = 0x83C167);
c!(YELLOW = 0xFFFF00);
c!(GOLD = 0xF0AC5F);
c!(RED = 0xFC6255);
c!(MAROON = 0xC55F73);
c!(PURPLE = 0x9A72AC);
c!(GREY = 0x888888);
