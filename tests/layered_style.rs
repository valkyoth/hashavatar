//! Alpha.4 layered-style capability, resolution, and parity assurance.

use hashavatar::{
    AccessoryStack, AvatarAccessory, AvatarAccessorySlot, AvatarBackground, AvatarError,
    AvatarExpression, AvatarKind, AvatarPalette, AvatarRequest, AvatarShape, AvatarStyle,
    LayoutDisposition, MAX_ACCESSORY_LAYERS, MAX_DIMENSION, StyleResolutionPolicy,
};
use sanitization_crypto_interop::sha2::SanitizedSha512;
use std::collections::BTreeSet;

fn prepare(
    kind: AvatarKind,
    style: AvatarStyle,
) -> Result<hashavatar::PreparedAvatar, AvatarError> {
    AvatarRequest::new(64, 64, 17, b"layered-style-fixture", style)
        .and_then(AvatarRequest::prepare)
        .inspect(|prepared| assert_eq!(prepared.resolved_style().kind(), kind))
}

fn base(kind: AvatarKind) -> AvatarStyle {
    AvatarStyle::new(kind, AvatarBackground::Themed, AvatarShape::Squircle)
}

#[test]
fn accessory_stack_capacity_fails_without_partial_growth() -> Result<(), AvatarError> {
    let mut stack = AccessoryStack::new();
    for accessory in [
        AvatarAccessory::Halo,
        AvatarAccessory::Hat,
        AvatarAccessory::Headphones,
        AvatarAccessory::Bowtie,
    ] {
        stack.try_push(accessory)?;
    }
    assert_eq!(stack.len(), MAX_ACCESSORY_LAYERS);
    assert_eq!(
        stack.try_push(AvatarAccessory::Glasses),
        Err(AvatarError::AccessoryCapacity {
            maximum: MAX_ACCESSORY_LAYERS
        })
    );
    assert_eq!(stack.len(), MAX_ACCESSORY_LAYERS);
    Ok(())
}

#[test]
fn strict_styles_reject_unsupported_duplicate_and_colliding_layers() -> Result<(), AvatarError> {
    let object = base(AvatarKind::Planet).with_accessory(AvatarAccessory::Hat)?;
    assert!(matches!(
        prepare(AvatarKind::Planet, object),
        Err(AvatarError::UnsupportedAccessory {
            accessory: AvatarAccessory::Hat
        })
    ));

    let duplicate = base(AvatarKind::Cat).with_accessories(AccessoryStack::from_slice(&[
        AvatarAccessory::Crown,
        AvatarAccessory::Hat,
    ])?);
    assert!(matches!(
        prepare(AvatarKind::Cat, duplicate),
        Err(AvatarError::AccessorySlotConflict {
            slot: AvatarAccessorySlot::Headwear
        })
    ));

    let collision = base(AvatarKind::Cat).with_accessories(AccessoryStack::from_slice(&[
        AvatarAccessory::Glasses,
        AvatarAccessory::Eyepatch,
    ])?);
    assert!(matches!(
        prepare(AvatarKind::Cat, collision),
        Err(AvatarError::AccessoryCollision {
            slot: AvatarAccessorySlot::Eyewear
        })
    ));
    Ok(())
}

#[test]
fn automatic_fallback_is_insertion_order_invariant() -> Result<(), AvatarError> {
    let first = AccessoryStack::from_slice(&[
        AvatarAccessory::Crown,
        AvatarAccessory::Hat,
        AvatarAccessory::Glasses,
        AvatarAccessory::Eyepatch,
    ])?;
    let second = AccessoryStack::from_slice(&[
        AvatarAccessory::Eyepatch,
        AvatarAccessory::Glasses,
        AvatarAccessory::Hat,
        AvatarAccessory::Crown,
    ])?;
    let make_style = |stack| {
        base(AvatarKind::Cat)
            .with_accessories(stack)
            .with_expression(AvatarExpression::Cool)
            .with_resolution_policy(StyleResolutionPolicy::AutomaticFallback)
    };
    let first = prepare(AvatarKind::Cat, make_style(first))?;
    let second = prepare(AvatarKind::Cat, make_style(second))?;
    assert_eq!(first.resolved_style(), second.resolved_style());
    assert_eq!(first.layout_report(), second.layout_report());
    assert_eq!(
        first.render_rgba()?.pixel_digest()?,
        second.render_rgba()?.pixel_digest()?
    );
    assert!(
        first
            .layout_report()
            .accessory_decisions()
            .any(|decision| matches!(decision.disposition(), LayoutDisposition::Substituted))
    );
    assert_eq!(
        first.layout_report().expression_decision().disposition(),
        LayoutDisposition::Substituted
    );
    Ok(())
}

#[test]
fn complete_layer_capability_matrix_is_explicit() -> Result<(), AvatarError> {
    for kind in AvatarKind::ALL {
        let capabilities = kind.capabilities();
        for palette in AvatarPalette::ALL {
            let prepared = prepare(kind, base(kind).with_palette(palette))?;
            assert_eq!(prepared.resolved_style().palette(), palette);
            assert!(capabilities.supports_palettes());
        }
        for expression in AvatarExpression::ALL {
            let result = prepare(kind, base(kind).with_expression(expression));
            if expression == AvatarExpression::Default || capabilities.supports_expressions() {
                let prepared = result?;
                assert_eq!(prepared.resolved_style().expression(), expression);
                assert!(roxmltree::Document::parse(&prepared.render_svg()?).is_ok());
            } else {
                assert!(matches!(
                    result,
                    Err(AvatarError::UnsupportedExpression { .. })
                ));
            }
        }
        for accessory in AvatarAccessory::ALL {
            let result = prepare(kind, base(kind).with_accessory(accessory)?);
            if capabilities.supports_accessory_slot(accessory.slot()) {
                let prepared = result?;
                assert_eq!(prepared.resolved_style().accessories().len(), 1);
                assert_eq!(prepared.render_rgba()?.dimensions(), (64, 64));
                assert!(roxmltree::Document::parse(&prepared.render_svg()?).is_ok());
            } else {
                assert!(matches!(
                    result,
                    Err(AvatarError::UnsupportedAccessory { .. })
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn every_face_family_accepts_a_maximum_noncolliding_stack() -> Result<(), AvatarError> {
    let stack = AccessoryStack::from_slice(&[
        AvatarAccessory::Halo,
        AvatarAccessory::Hat,
        AvatarAccessory::Headphones,
        AvatarAccessory::Bowtie,
    ])?;
    for kind in AvatarKind::ALL {
        if !kind.capabilities().has_face_anchors() {
            continue;
        }
        for dimension in [64, 256, MAX_DIMENSION] {
            let style = AvatarStyle::new(kind, AvatarBackground::PolkaDot, AvatarShape::Octagon)
                .with_palette(AvatarPalette::Gold)
                .with_expression(AvatarExpression::Crying)
                .with_accessories(stack);
            let prepared =
                AvatarRequest::new(dimension, dimension, 19, b"layered-family-stress", style)?
                    .prepare()?;
            assert_eq!(
                prepared.layout_report().accessory_decision_count(),
                MAX_ACCESSORY_LAYERS
            );
            assert_eq!(
                prepared.resolved_style().accessories().len(),
                MAX_ACCESSORY_LAYERS
            );
            assert!(prepared.scene_report().command_count() <= 64);
            if dimension == 64 {
                assert_eq!(prepared.render_rgba()?.dimensions(), (64, 64));
                assert!(roxmltree::Document::parse(&prepared.render_svg()?).is_ok());
            }
        }
    }
    Ok(())
}

#[test]
fn automatic_object_layers_are_reported_not_silently_skipped() -> Result<(), AvatarError> {
    let prepared = prepare(
        AvatarKind::Shield,
        AvatarStyle::automatic(
            AvatarKind::Shield,
            AvatarBackground::Ocean,
            AvatarShape::Circle,
        ),
    )?;
    assert!(prepared.resolved_style().automatically_derived());
    assert!(prepared.resolved_style().accessories().is_empty());
    assert!(
        prepared
            .layout_report()
            .accessory_decisions()
            .all(|decision| matches!(decision.disposition(), LayoutDisposition::Rejected))
    );
    assert_eq!(
        prepared.layout_report().expression_decision().effective(),
        AvatarExpression::Default
    );
    Ok(())
}

#[test]
fn representative_layers_and_palettes_have_distinct_pixels() -> Result<(), AvatarError> {
    let mut palette_digests = BTreeSet::new();
    let mut palette_roles = BTreeSet::new();
    for palette in AvatarPalette::ALL {
        let prepared = prepare(AvatarKind::Cat, base(AvatarKind::Cat).with_palette(palette))?;
        let roles = prepared.resolved_style().color_roles();
        assert!(palette_roles.insert((
            roles.primary().channels(),
            roles.secondary().channels(),
            roles.accent().channels(),
        )));
        assert!(palette_digests.insert(*prepared.render_rgba()?.pixel_digest()?.as_bytes()));
    }

    let mut accessory_digests = BTreeSet::new();
    for accessory in AvatarAccessory::ALL {
        let prepared = prepare(
            AvatarKind::Cat,
            base(AvatarKind::Cat).with_accessory(accessory)?,
        )?;
        assert!(accessory_digests.insert(*prepared.render_rgba()?.pixel_digest()?.as_bytes()));
    }

    let mut expression_digests = BTreeSet::new();
    for expression in AvatarExpression::ALL {
        let prepared = prepare(
            AvatarKind::Cat,
            base(AvatarKind::Cat).with_expression(expression),
        )?;
        assert!(
            expression_digests.insert(*prepared.render_rgba()?.pixel_digest()?.as_bytes()),
            "duplicate expression pixels: {}",
            expression.as_str()
        );
    }
    Ok(())
}

#[test]
fn complete_face_layer_corpus_is_distinct_and_stable() -> Result<(), AvatarError> {
    let mut aggregate = SanitizedSha512::new();
    for kind in AvatarKind::ALL {
        if !kind.capabilities().has_face_anchors() {
            continue;
        }

        let mut expression_digests = BTreeSet::new();
        for expression in AvatarExpression::ALL {
            let prepared = prepare(kind, base(kind).with_expression(expression))?;
            let digest = prepared.render_rgba()?.pixel_digest()?;
            assert!(
                expression_digests.insert(*digest.as_bytes()),
                "duplicate {} expression pixels: {}",
                kind.as_str(),
                expression.as_str()
            );
            aggregate.update(digest.as_bytes());
        }

        let baseline = prepare(kind, base(kind))?.render_rgba()?.pixel_digest()?;
        let mut accessory_digests = BTreeSet::from([*baseline.as_bytes()]);
        aggregate.update(baseline.as_bytes());
        for accessory in AvatarAccessory::ALL {
            let prepared = prepare(kind, base(kind).with_accessory(accessory)?)?;
            let digest = prepared.render_rgba()?.pixel_digest()?;
            assert!(
                accessory_digests.insert(*digest.as_bytes()),
                "duplicate {} accessory pixels: {}",
                kind.as_str(),
                accessory.as_str()
            );
            aggregate.update(digest.as_bytes());
        }
    }

    assert_eq!(
        aggregate.finalize(),
        [
            171, 11, 200, 239, 95, 60, 69, 59, 100, 101, 55, 22, 193, 61, 92, 213, 215, 104, 54,
            19, 243, 245, 202, 135, 143, 79, 247, 92, 226, 222, 206, 24, 136, 33, 130, 103, 30,
            157, 35, 99, 34, 94, 248, 145, 3, 11, 117, 221, 185, 23, 105, 3, 244, 208, 107, 151,
            166, 23, 85, 38, 222, 26, 236, 103,
        ]
    );
    Ok(())
}
