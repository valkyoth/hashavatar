use super::{
    AccessoryLayoutDecision, ExpressionLayoutDecision, LayoutDisposition, LayoutReport,
    ResolvedStyle, accessory_offset_basis_points, accessory_z_band, anchors_for, is_automatic,
    slots_collide,
};
use crate::{
    AccessoryStack, AvatarAccessory, AvatarError, AvatarExpression, AvatarStyle, AvatarTraitVector,
    MAX_ACCESSORY_LAYERS, StyleResolutionPolicy, style::resolve_color_roles,
};

pub(crate) fn resolve_style(
    style: AvatarStyle,
    traits: AvatarTraitVector,
) -> Result<(ResolvedStyle, LayoutReport), AvatarError> {
    let automatically_derived = is_automatic(style.selection());
    let mut requested = if automatically_derived {
        let mut stack = AccessoryStack::new();
        stack.try_push(AvatarAccessory::from_sample(traits.detail_a()))?;
        stack.try_push(AvatarAccessory::from_sample(traits.detail_b()))?;
        stack
    } else {
        style.accessories()
    };
    requested
        .as_mut_slice()
        .sort_by_key(|accessory| (accessory.slot().catalog_id(), accessory.catalog_id()));

    let palette = if automatically_derived {
        crate::AvatarPalette::from_sample(traits.accent_hue())
    } else {
        style.palette()
    };
    let requested_expression = if automatically_derived {
        AvatarExpression::from_sample(traits.pattern_seed())
    } else {
        style.expression()
    };
    let anchors = anchors_for(style.kind());
    let policy = style.resolution_policy();
    let mut occupied = 0_u16;
    let mut decisions = [None; MAX_ACCESSORY_LAYERS];
    let mut decision_count = 0_usize;

    for accessory in requested.iter() {
        let direct = admit(accessory, anchors.is_some(), occupied);
        let (effective, disposition) = if direct {
            (Some(accessory), LayoutDisposition::Accepted)
        } else if matches!(policy, StyleResolutionPolicy::Strict) {
            return Err(strict_accessory_error(
                accessory,
                anchors.is_some(),
                occupied,
            ));
        } else if let Some(fallback) = automatic_fallback(anchors.is_some(), occupied) {
            (Some(fallback), LayoutDisposition::Substituted)
        } else {
            (None, LayoutDisposition::Rejected)
        };
        if let Some(effective) = effective {
            occupied |= effective.slot().bit();
        }
        push_decision(
            &mut decisions,
            &mut decision_count,
            AccessoryLayoutDecision {
                requested: accessory,
                effective,
                disposition,
                z_band: accessory_z_band(effective.unwrap_or(accessory)),
                vertical_adjustment_basis_points: 0,
            },
        )?;
    }

    apply_adjustments(style.kind(), occupied, &mut decisions);
    let expression = resolve_expression(requested_expression, anchors.is_some(), occupied, policy)?;
    decisions
        .get_mut(..decision_count)
        .ok_or(AvatarError::InvalidScene)?
        .sort_by_key(|decision| decision_sort_key(*decision));

    let mut effective_accessories = AccessoryStack::new();
    for decision in decisions.iter().filter_map(|decision| *decision) {
        if let Some(accessory) = decision.effective {
            effective_accessories.try_push(accessory)?;
        }
    }
    let resolved = ResolvedStyle {
        kind: style.kind(),
        background: style.background(),
        shape: style.shape(),
        palette,
        colors: resolve_color_roles(palette, traits, style.kind()),
        expression: expression.effective,
        accessories: effective_accessories,
        automatically_derived,
    };
    Ok((
        resolved,
        LayoutReport {
            policy,
            anchors,
            requested_accessories: requested,
            decisions,
            decision_count: u8::try_from(decision_count).map_err(|_| AvatarError::NumericRange)?,
            expression,
        },
    ))
}

fn admit(accessory: AvatarAccessory, has_anchors: bool, occupied: u16) -> bool {
    has_anchors
        && occupied & accessory.slot().bit() == 0
        && !occupied_slots_collide(accessory, occupied)
}

fn occupied_slots_collide(accessory: AvatarAccessory, occupied: u16) -> bool {
    crate::AvatarAccessorySlot::ALL_ADMITTED
        .iter()
        .copied()
        .any(|slot| occupied & slot.bit() != 0 && slots_collide(accessory.slot(), slot))
}

fn automatic_fallback(has_anchors: bool, occupied: u16) -> Option<AvatarAccessory> {
    if !has_anchors {
        return None;
    }
    AvatarAccessory::ALL
        .iter()
        .copied()
        .find(|candidate| admit(*candidate, true, occupied))
}

fn strict_accessory_error(
    accessory: AvatarAccessory,
    has_anchors: bool,
    occupied: u16,
) -> AvatarError {
    if !has_anchors {
        AvatarError::UnsupportedAccessory { accessory }
    } else if occupied & accessory.slot().bit() != 0 {
        AvatarError::AccessorySlotConflict {
            slot: accessory.slot(),
        }
    } else {
        AvatarError::AccessoryCollision {
            slot: accessory.slot(),
        }
    }
}

fn push_decision(
    decisions: &mut [Option<AccessoryLayoutDecision>; MAX_ACCESSORY_LAYERS],
    count: &mut usize,
    decision: AccessoryLayoutDecision,
) -> Result<(), AvatarError> {
    let slot = decisions
        .get_mut(*count)
        .ok_or(AvatarError::AccessoryCapacity {
            maximum: MAX_ACCESSORY_LAYERS,
        })?;
    *slot = Some(decision);
    *count = count.checked_add(1).ok_or(AvatarError::NumericRange)?;
    Ok(())
}

fn apply_adjustments(
    kind: crate::AvatarKind,
    occupied: u16,
    decisions: &mut [Option<AccessoryLayoutDecision>; MAX_ACCESSORY_LAYERS],
) {
    let has_headwear = occupied & crate::AvatarAccessorySlot::Headwear.bit() != 0;
    let has_earwear = occupied & crate::AvatarAccessorySlot::Earwear.bit() != 0;
    for decision in decisions.iter_mut().filter_map(Option::as_mut) {
        let Some(accessory) = decision.effective else {
            continue;
        };
        let mut adjustment = accessory_offset_basis_points(kind, accessory);
        if matches!(accessory, AvatarAccessory::Halo) && has_headwear {
            adjustment = adjustment.saturating_sub(600);
        }
        if matches!(accessory.slot(), crate::AvatarAccessorySlot::Headwear) && has_earwear {
            adjustment = adjustment.saturating_sub(400);
        }
        decision.vertical_adjustment_basis_points = adjustment;
        if adjustment != 0 && matches!(decision.disposition, LayoutDisposition::Accepted) {
            decision.disposition = LayoutDisposition::Adjusted;
        }
    }
}

fn resolve_expression(
    requested: AvatarExpression,
    has_anchors: bool,
    occupied: u16,
    policy: StyleResolutionPolicy,
) -> Result<ExpressionLayoutDecision, AvatarError> {
    let unsupported = !matches!(requested, AvatarExpression::Default) && !has_anchors;
    let cool_collision = matches!(requested, AvatarExpression::Cool)
        && occupied
            & (crate::AvatarAccessorySlot::Eyewear.bit()
                | crate::AvatarAccessorySlot::Facewear.bit())
            != 0;
    if (unsupported || cool_collision) && matches!(policy, StyleResolutionPolicy::Strict) {
        return if unsupported {
            Err(AvatarError::UnsupportedExpression {
                expression: requested,
            })
        } else {
            Err(AvatarError::ExpressionCollision {
                expression: requested,
            })
        };
    }
    let effective = if unsupported {
        AvatarExpression::Default
    } else if cool_collision {
        AvatarExpression::Happy
    } else {
        requested
    };
    Ok(ExpressionLayoutDecision {
        requested,
        effective,
        disposition: if effective == requested {
            LayoutDisposition::Accepted
        } else {
            LayoutDisposition::Substituted
        },
    })
}

fn decision_sort_key(decision: Option<AccessoryLayoutDecision>) -> (u8, u8, u16, u8) {
    let Some(decision) = decision else {
        return (u8::MAX, u8::MAX, u16::MAX, u8::MAX);
    };
    let accessory = decision.effective.unwrap_or(decision.requested);
    (
        decision.z_band as u8,
        accessory.slot().catalog_id(),
        accessory.catalog_id(),
        u8::from(decision.effective.is_none()),
    )
}
