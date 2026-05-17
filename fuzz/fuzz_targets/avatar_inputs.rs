#![no_main]

use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_for_id, render_avatar_svg_for_id,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let kind = avatar_kind(data[0]);
    let background = avatar_background(*data.get(1).unwrap_or(&0));
    let size = 64 + u32::from(*data.get(2).unwrap_or(&0) % 8) * 64;
    let identity_len = data.len().min(128);
    let identity = &data[..identity_len];
    let Ok(spec) = AvatarSpec::new(size, size, 0) else {
        return;
    };
    let options = AvatarOptions::new(kind, background);

    let _ = render_avatar_svg_for_id(spec, identity, options);
    let _ = encode_avatar_for_id(spec, identity, AvatarOutputFormat::Png, options);
});

fn avatar_kind(value: u8) -> AvatarKind {
    match value % 23 {
        0 => AvatarKind::Cat,
        1 => AvatarKind::Dog,
        2 => AvatarKind::Robot,
        3 => AvatarKind::Fox,
        4 => AvatarKind::Alien,
        5 => AvatarKind::Monster,
        6 => AvatarKind::Ghost,
        7 => AvatarKind::Slime,
        8 => AvatarKind::Bird,
        9 => AvatarKind::Wizard,
        10 => AvatarKind::Skull,
        11 => AvatarKind::Paws,
        12 => AvatarKind::Planet,
        13 => AvatarKind::Rocket,
        14 => AvatarKind::Mushroom,
        15 => AvatarKind::Cactus,
        16 => AvatarKind::Frog,
        17 => AvatarKind::Panda,
        18 => AvatarKind::Cupcake,
        19 => AvatarKind::Pizza,
        20 => AvatarKind::Icecream,
        21 => AvatarKind::Octopus,
        _ => AvatarKind::Knight,
    }
}

fn avatar_background(value: u8) -> AvatarBackground {
    match value % 6 {
        0 => AvatarBackground::Themed,
        1 => AvatarBackground::White,
        2 => AvatarBackground::Black,
        3 => AvatarBackground::Dark,
        4 => AvatarBackground::Light,
        _ => AvatarBackground::Transparent,
    }
}
