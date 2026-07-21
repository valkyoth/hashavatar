mod common;
mod creatures;
mod faces;
mod objects;

use self::common::FamilyRig;
use crate::{AvatarKind, AvatarTraitVector, CatError, scene::Scene};

pub(super) fn compile(
    scene: &mut Scene,
    kind: AvatarKind,
    traits: AvatarTraitVector,
) -> Result<(), CatError> {
    let rig = FamilyRig::new(scene, traits)?;
    match kind {
        AvatarKind::Cat
        | AvatarKind::Dog
        | AvatarKind::Robot
        | AvatarKind::Fox
        | AvatarKind::Alien
        | AvatarKind::Monster
        | AvatarKind::Wizard
        | AvatarKind::Skull
        | AvatarKind::Frog
        | AvatarKind::Panda
        | AvatarKind::Knight
        | AvatarKind::Bear
        | AvatarKind::Dragon
        | AvatarKind::Ninja
        | AvatarKind::Astronaut => faces::compile(scene, kind, rig),
        AvatarKind::Ghost
        | AvatarKind::Slime
        | AvatarKind::Bird
        | AvatarKind::Octopus
        | AvatarKind::Penguin => creatures::compile(scene, kind, rig),
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
        | AvatarKind::Shield => objects::compile(scene, kind, rig),
    }
}
