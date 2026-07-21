mod anchors;
mod resolve;

pub(crate) use self::{
    anchors::{accessory_offset_basis_points, anchors_for},
    resolve::resolve_style,
};

use crate::{
    AccessoryStack, AvatarAccessory, AvatarAccessorySlot, AvatarBackground, AvatarColorRoles,
    AvatarExpression, AvatarKind, AvatarPalette, AvatarShape, StyleResolutionPolicy,
    style::LayerSelection,
};

/// One family-calibrated point expressed in basis points of output size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarAnchorPoint {
    x_basis_points: u16,
    y_basis_points: u16,
}

impl AvatarAnchorPoint {
    pub(crate) const fn new(x_basis_points: u16, y_basis_points: u16) -> Self {
        Self {
            x_basis_points,
            y_basis_points,
        }
    }

    /// Returns horizontal position in basis points (`0..=10_000`).
    pub const fn x_basis_points(self) -> u16 {
        self.x_basis_points
    }

    /// Returns vertical position in basis points (`0..=10_000`).
    pub const fn y_basis_points(self) -> u16 {
        self.y_basis_points
    }
}

/// Family-calibrated semantic face anchors and safe spans.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvatarAnchorSet {
    left_eye: AvatarAnchorPoint,
    right_eye: AvatarAnchorPoint,
    mouth: AvatarAnchorPoint,
    crown: AvatarAnchorPoint,
    neck: AvatarAnchorPoint,
    face_width_basis_points: u16,
    eye_radius_basis_points: u16,
}

impl AvatarAnchorSet {
    /// Returns the left-eye anchor.
    pub const fn left_eye(self) -> AvatarAnchorPoint {
        self.left_eye
    }
    /// Returns the right-eye anchor.
    pub const fn right_eye(self) -> AvatarAnchorPoint {
        self.right_eye
    }
    /// Returns the mouth anchor.
    pub const fn mouth(self) -> AvatarAnchorPoint {
        self.mouth
    }
    /// Returns the crown/head-top anchor.
    pub const fn crown(self) -> AvatarAnchorPoint {
        self.crown
    }
    /// Returns the neck anchor.
    pub const fn neck(self) -> AvatarAnchorPoint {
        self.neck
    }
    /// Returns the calibrated face width in basis points of the smaller side.
    pub const fn face_width_basis_points(self) -> u16 {
        self.face_width_basis_points
    }
    /// Returns the eye radius in basis points of the smaller side.
    pub const fn eye_radius_basis_points(self) -> u16 {
        self.eye_radius_basis_points
    }
}

/// Result assigned to one requested visual layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayoutDisposition {
    /// The requested layer was accepted unchanged.
    Accepted,
    /// The requested layer was accepted with a calibrated transform.
    Adjusted,
    /// Automatic fallback replaced the requested layer.
    Substituted,
    /// Automatic fallback could not admit the requested layer.
    Rejected,
}

/// Stable semantic drawing band used to order layered composition.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AvatarZBand {
    /// Behind the family subject.
    BehindSubject,
    /// Expression overlays above the family subject.
    Expression,
    /// Foreground accessories.
    Foreground,
}

/// Resolution record for one requested accessory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessoryLayoutDecision {
    requested: AvatarAccessory,
    effective: Option<AvatarAccessory>,
    disposition: LayoutDisposition,
    z_band: AvatarZBand,
    vertical_adjustment_basis_points: i16,
}

impl AccessoryLayoutDecision {
    /// Returns the caller-requested accessory.
    pub const fn requested(self) -> AvatarAccessory {
        self.requested
    }
    /// Returns the effective accessory, or `None` when rejected.
    pub const fn effective(self) -> Option<AvatarAccessory> {
        self.effective
    }
    /// Returns the resolution disposition.
    pub const fn disposition(self) -> LayoutDisposition {
        self.disposition
    }
    /// Returns the canonical drawing band.
    pub const fn z_band(self) -> AvatarZBand {
        self.z_band
    }
    /// Returns the signed vertical adjustment in basis points.
    pub const fn vertical_adjustment_basis_points(self) -> i16 {
        self.vertical_adjustment_basis_points
    }
}

/// Resolution record for the expression layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExpressionLayoutDecision {
    requested: AvatarExpression,
    effective: AvatarExpression,
    disposition: LayoutDisposition,
}

impl ExpressionLayoutDecision {
    /// Returns the requested expression.
    pub const fn requested(self) -> AvatarExpression {
        self.requested
    }
    /// Returns the effective expression.
    pub const fn effective(self) -> AvatarExpression {
        self.effective
    }
    /// Returns the resolution disposition.
    pub const fn disposition(self) -> LayoutDisposition {
        self.disposition
    }
}

/// Immutable effective style consumed by canonical scene compilation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedStyle {
    kind: AvatarKind,
    background: AvatarBackground,
    shape: AvatarShape,
    palette: AvatarPalette,
    colors: AvatarColorRoles,
    expression: AvatarExpression,
    accessories: AccessoryStack,
    automatically_derived: bool,
}

impl ResolvedStyle {
    /// Returns the effective family.
    pub const fn kind(self) -> AvatarKind {
        self.kind
    }
    /// Returns the effective background.
    pub const fn background(self) -> AvatarBackground {
        self.background
    }
    /// Returns the effective frame shape.
    pub const fn shape(self) -> AvatarShape {
        self.shape
    }
    /// Returns the effective palette.
    pub const fn palette(self) -> AvatarPalette {
        self.palette
    }
    /// Returns resolved integer color roles.
    pub const fn color_roles(self) -> AvatarColorRoles {
        self.colors
    }
    /// Returns the effective expression.
    pub const fn expression(self) -> AvatarExpression {
        self.expression
    }
    /// Returns effective accessories in canonical drawing order.
    pub const fn accessories(self) -> AccessoryStack {
        self.accessories
    }
    /// Returns whether optional layers were identity-derived.
    pub const fn automatically_derived(self) -> bool {
        self.automatically_derived
    }
}

/// Complete deterministic style-resolution and placement report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutReport {
    policy: StyleResolutionPolicy,
    anchors: Option<AvatarAnchorSet>,
    requested_accessories: AccessoryStack,
    decisions: [Option<AccessoryLayoutDecision>; crate::MAX_ACCESSORY_LAYERS],
    decision_count: u8,
    expression: ExpressionLayoutDecision,
}

impl LayoutReport {
    /// Returns the applied compatibility policy.
    pub const fn resolution_policy(self) -> StyleResolutionPolicy {
        self.policy
    }
    /// Returns calibrated anchors when the family supports face layers.
    pub const fn anchors(self) -> Option<AvatarAnchorSet> {
        self.anchors
    }
    /// Returns the canonicalized requested accessory stack.
    pub const fn requested_accessories(self) -> AccessoryStack {
        self.requested_accessories
    }
    /// Returns the number of accessory decisions.
    pub const fn accessory_decision_count(self) -> usize {
        self.decision_count as usize
    }
    /// Iterates over accessory decisions in canonical drawing order.
    pub fn accessory_decisions(&self) -> impl Iterator<Item = AccessoryLayoutDecision> + '_ {
        self.decisions.iter().filter_map(|decision| *decision)
    }
    /// Returns expression resolution information.
    pub const fn expression_decision(self) -> ExpressionLayoutDecision {
        self.expression
    }
}

pub(crate) const fn accessory_z_band(accessory: AvatarAccessory) -> AvatarZBand {
    match accessory {
        AvatarAccessory::Halo | AvatarAccessory::Horns => AvatarZBand::BehindSubject,
        AvatarAccessory::Glasses
        | AvatarAccessory::Hat
        | AvatarAccessory::Headphones
        | AvatarAccessory::Crown
        | AvatarAccessory::Bowtie
        | AvatarAccessory::Eyepatch
        | AvatarAccessory::Scarf => AvatarZBand::Foreground,
    }
}

pub(crate) const fn slots_collide(first: AvatarAccessorySlot, second: AvatarAccessorySlot) -> bool {
    matches!(
        (first, second),
        (AvatarAccessorySlot::Facewear, AvatarAccessorySlot::Eyewear)
            | (AvatarAccessorySlot::Eyewear, AvatarAccessorySlot::Facewear)
    )
}

pub(crate) const fn is_automatic(selection: LayerSelection) -> bool {
    matches!(selection, LayerSelection::Automatic)
}
