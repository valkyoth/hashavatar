use crate::{AvatarAccessory, AvatarAnchorPoint, AvatarAnchorSet, AvatarKind};

const fn anchors(
    left_eye: (u16, u16),
    right_eye: (u16, u16),
    mouth: (u16, u16),
    top: u16,
    neck: u16,
    face_width: u16,
    eye_radius: u16,
) -> AvatarAnchorSet {
    AvatarAnchorSet {
        left_eye: AvatarAnchorPoint::new(left_eye.0, left_eye.1),
        right_eye: AvatarAnchorPoint::new(right_eye.0, right_eye.1),
        mouth: AvatarAnchorPoint::new(mouth.0, mouth.1),
        crown: AvatarAnchorPoint::new(5_000, top),
        neck: AvatarAnchorPoint::new(5_000, neck),
        face_width_basis_points: face_width,
        eye_radius_basis_points: eye_radius,
    }
}

pub(crate) const fn anchors_for(kind: AvatarKind) -> Option<AvatarAnchorSet> {
    Some(match kind {
        AvatarKind::Cat => anchors(
            (3_900, 5_100),
            (6_100, 5_100),
            (5_000, 7_000),
            2_800,
            7_200,
            5_800,
            550,
        ),
        AvatarKind::Dog => anchors(
            (3_900, 5_000),
            (6_100, 5_000),
            (5_000, 6_900),
            2_400,
            7_200,
            6_200,
            550,
        ),
        AvatarKind::Robot => anchors(
            (3_900, 4_800),
            (6_100, 4_800),
            (5_000, 7_000),
            2_500,
            7_200,
            5_400,
            600,
        ),
        AvatarKind::Fox => anchors(
            (3_900, 4_900),
            (6_100, 4_900),
            (5_000, 6_900),
            2_500,
            7_200,
            6_000,
            500,
        ),
        AvatarKind::Alien => anchors(
            (4_100, 4_700),
            (5_900, 4_700),
            (5_000, 6_500),
            2_500,
            7_200,
            5_800,
            700,
        ),
        AvatarKind::Monster => anchors(
            (4_000, 5_100),
            (6_000, 5_100),
            (5_000, 7_100),
            2_700,
            7_400,
            6_200,
            450,
        ),
        AvatarKind::Ghost => anchors(
            (4_400, 5_000),
            (5_600, 5_000),
            (5_000, 6_500),
            2_700,
            7_200,
            5_400,
            450,
        ),
        AvatarKind::Slime => anchors(
            (4_000, 4_800),
            (6_000, 4_800),
            (5_000, 6_400),
            3_200,
            7_300,
            5_800,
            550,
        ),
        AvatarKind::Bird => anchors(
            (4_450, 5_100),
            (5_550, 5_100),
            (5_000, 6_100),
            3_100,
            7_000,
            5_000,
            350,
        ),
        AvatarKind::Wizard => anchors(
            (4_550, 5_500),
            (5_450, 5_500),
            (5_000, 6_700),
            3_600,
            7_300,
            4_400,
            400,
        ),
        AvatarKind::Skull => anchors(
            (3_900, 5_000),
            (6_100, 5_000),
            (5_000, 6_800),
            2_400,
            7_200,
            5_500,
            650,
        ),
        AvatarKind::Frog => anchors(
            (3_700, 3_900),
            (6_300, 3_900),
            (5_000, 6_200),
            2_500,
            7_200,
            6_400,
            700,
        ),
        AvatarKind::Panda => anchors(
            (3_900, 5_100),
            (6_100, 5_100),
            (5_000, 6_700),
            2_800,
            7_200,
            6_200,
            400,
        ),
        AvatarKind::Octopus => anchors(
            (4_200, 5_200),
            (5_800, 5_200),
            (5_000, 6_300),
            2_800,
            7_000,
            5_800,
            450,
        ),
        AvatarKind::Knight => anchors(
            (4_000, 4_500),
            (6_000, 4_500),
            (5_000, 6_800),
            2_200,
            7_200,
            5_800,
            500,
        ),
        AvatarKind::Bear => anchors(
            (3_900, 4_800),
            (6_100, 4_800),
            (5_000, 6_700),
            2_500,
            7_300,
            6_200,
            550,
        ),
        AvatarKind::Penguin => anchors(
            (4_000, 4_500),
            (6_000, 4_500),
            (5_000, 5_700),
            2_000,
            7_400,
            5_800,
            450,
        ),
        AvatarKind::Dragon => anchors(
            (3_900, 4_900),
            (6_100, 4_900),
            (5_000, 7_200),
            2_300,
            7_300,
            6_200,
            520,
        ),
        AvatarKind::Ninja => anchors(
            (4_000, 4_800),
            (6_000, 4_800),
            (5_000, 6_400),
            2_400,
            7_300,
            5_800,
            450,
        ),
        AvatarKind::Astronaut => anchors(
            (4_000, 4_700),
            (6_000, 4_700),
            (5_000, 6_200),
            1_900,
            7_600,
            6_000,
            450,
        ),
        AvatarKind::Paws
        | AvatarKind::Planet
        | AvatarKind::Rocket
        | AvatarKind::Mushroom
        | AvatarKind::Cactus
        | AvatarKind::Cupcake
        | AvatarKind::Pizza
        | AvatarKind::Icecream
        | AvatarKind::Diamond
        | AvatarKind::CoffeeCup
        | AvatarKind::Shield => return None,
    })
}

pub(crate) const fn accessory_offset_basis_points(
    kind: AvatarKind,
    accessory: AvatarAccessory,
) -> i16 {
    match accessory {
        AvatarAccessory::Glasses => match kind {
            AvatarKind::Ghost => 100,
            AvatarKind::Wizard | AvatarKind::Knight => 350,
            _ => 0,
        },
        AvatarAccessory::Hat => match kind {
            AvatarKind::Cat | AvatarKind::Frog => -350,
            _ => 0,
        },
        AvatarAccessory::Crown => match kind {
            AvatarKind::Cat => -350,
            AvatarKind::Alien | AvatarKind::Frog => -800,
            _ => 0,
        },
        AvatarAccessory::Bowtie => match kind {
            AvatarKind::Cat
            | AvatarKind::Fox
            | AvatarKind::Slime
            | AvatarKind::Wizard
            | AvatarKind::Octopus => 600,
            _ => 0,
        },
        AvatarAccessory::Eyepatch if matches!(kind, AvatarKind::Knight) => 300,
        AvatarAccessory::Horns if matches!(kind, AvatarKind::Dog | AvatarKind::Robot) => 700,
        AvatarAccessory::Headphones
        | AvatarAccessory::Eyepatch
        | AvatarAccessory::Scarf
        | AvatarAccessory::Halo
        | AvatarAccessory::Horns => 0,
    }
}
