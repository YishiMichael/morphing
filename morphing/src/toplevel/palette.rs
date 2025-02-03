macro_rules! c {
    ($value:tt as $name:ident) => {
        pub const $name: ::palette::Srgb<f32> = ::palette::Srgb::new(
            ($value as u32 >> 16 & 0xFF) as f32 / 0xFF as f32,
            ($value as u32 >> 8 & 0xFF) as f32 / 0xFF as f32,
            ($value as u32 & 0xFF) as f32 / 0xFF as f32,
        );
    };
    (use $alias:ident as $name:ident) => {
        pub const $name: ::palette::Srgb<f32> = $alias;
    };
}

c!(0x1C758A as BLUE_E);
c!(0x29ABCA as BLUE_D);
c!(0x58C4DD as BLUE_C);
c!(0x9CDCEB as BLUE_B);
c!(0xC7E9F1 as BLUE_A);
c!(0x49A88F as TEAL_E);
c!(0x55C1A7 as TEAL_D);
c!(0x5CD0B3 as TEAL_C);
c!(0x76DDC0 as TEAL_B);
c!(0xACEAD7 as TEAL_A);
c!(0x699C52 as GREEN_E);
c!(0x77B05D as GREEN_D);
c!(0x83C167 as GREEN_C);
c!(0xA6CF8C as GREEN_B);
c!(0xC9E2AE as GREEN_A);
c!(0xE8C11C as YELLOW_E);
c!(0xF4D345 as YELLOW_D);
c!(0xFFFF00 as YELLOW_C);
c!(0xFFEA94 as YELLOW_B);
c!(0xFFF1B6 as YELLOW_A);
c!(0xC78D46 as GOLD_E);
c!(0xE1A158 as GOLD_D);
c!(0xF0AC5F as GOLD_C);
c!(0xF9B775 as GOLD_B);
c!(0xF7C797 as GOLD_A);
c!(0xCF5044 as RED_E);
c!(0xE65A4C as RED_D);
c!(0xFC6255 as RED_C);
c!(0xFF8080 as RED_B);
c!(0xF7A1A3 as RED_A);
c!(0x94424F as MAROON_E);
c!(0xA24D61 as MAROON_D);
c!(0xC55F73 as MAROON_C);
c!(0xEC92AB as MAROON_B);
c!(0xECABC1 as MAROON_A);
c!(0x644172 as PURPLE_E);
c!(0x715582 as PURPLE_D);
c!(0x9A72AC as PURPLE_C);
c!(0xB189C6 as PURPLE_B);
c!(0xCAA3E8 as PURPLE_A);
c!(0x222222 as GREY_E);
c!(0x444444 as GREY_D);
c!(0x888888 as GREY_C);
c!(0xBBBBBB as GREY_B);
c!(0xDDDDDD as GREY_A);
c!(0xFFFFFF as WHITE);
c!(0x000000 as BLACK);
c!(0x736357 as GREY_BROWN);
c!(0x8B4513 as DARK_BROWN);
c!(0xCD853F as LIGHT_BROWN);
c!(0xD147BD as PINK);
c!(0xDC75CD as LIGHT_PINK);
c!(0x00FF00 as GREEN_SCREEN);
c!(0xFF862F as ORANGE);

c!(use BLUE_C as BLUE);
c!(use TEAL_C as TEAL);
c!(use GREEN_C as GREEN);
c!(use YELLOW_C as YELLOW);
c!(use GOLD_C as GOLD);
c!(use RED_C as RED);
c!(use MAROON_C as MAROON);
c!(use PURPLE_C as PURPLE);
c!(use GREY_C as GREY);
