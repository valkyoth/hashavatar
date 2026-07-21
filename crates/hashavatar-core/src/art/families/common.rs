use crate::{
    AvatarColorRoles, AvatarError, AvatarTraitVector,
    art::util::{Canvas, role_color, vary},
    fixed::Fixed,
    geometry::{FillRule, Path, Point, Rect},
    paint::{Color, Paint},
    scene::{Command, Scene, Stroke},
};

#[derive(Clone, Copy)]
pub(super) struct FamilyRig {
    pub(super) canvas: Canvas,
    pub(super) traits: AvatarTraitVector,
    pub(super) primary: Color,
    pub(super) secondary: Color,
    pub(super) accent: Color,
    pub(super) light: Color,
    pub(super) ink: Color,
}

impl FamilyRig {
    pub(super) fn new(
        scene: &Scene,
        traits: AvatarTraitVector,
        colors: AvatarColorRoles,
    ) -> Result<Self, AvatarError> {
        Ok(Self {
            canvas: Canvas::new(scene)?,
            traits,
            primary: role_color(colors.primary()),
            secondary: role_color(colors.secondary()),
            accent: role_color(colors.accent()),
            light: role_color(colors.light()),
            ink: role_color(colors.ink()),
        })
    }

    pub(super) fn head_rx(self) -> Result<Fixed, AvatarError> {
        vary(self.canvas.minimum, 24, 30, self.traits.proportion_a())
    }

    pub(super) fn head_ry(self) -> Result<Fixed, AvatarError> {
        vary(self.canvas.minimum, 23, 30, self.traits.proportion_b())
    }
}

pub(super) fn ellipse(
    scene: &mut Scene,
    center: Point,
    radius_x: Fixed,
    radius_y: Fixed,
    color: Color,
) -> Result<(), AvatarError> {
    scene.push(Command::Ellipse {
        center,
        radius_x,
        radius_y,
        paint: Paint::solid(color),
    })
}

pub(super) fn rect(scene: &mut Scene, rect: Rect, color: Color) -> Result<(), AvatarError> {
    scene.push(Command::Rect {
        rect,
        paint: Paint::solid(color),
    })
}

pub(super) fn triangle(
    scene: &mut Scene,
    points: [Point; 3],
    color: Color,
) -> Result<(), AvatarError> {
    scene.push(Command::Triangle {
        points,
        paint: Paint::solid(color),
    })
}

pub(super) fn line(
    scene: &mut Scene,
    start: Point,
    end: Point,
    width: Fixed,
    color: Color,
) -> Result<(), AvatarError> {
    scene.push(Command::Line {
        start,
        end,
        stroke: Stroke {
            width,
            paint: Paint::solid(color),
        },
    })
}

pub(super) fn polygon(
    scene: &mut Scene,
    rig: FamilyRig,
    points: &[(i32, i32)],
    color: Color,
) -> Result<(), AvatarError> {
    let first = points.first().ok_or(AvatarError::InvalidScene)?;
    let mut path = Path::builder(Point::new(rig.canvas.x(first.0)?, rig.canvas.y(first.1)?))?;
    for point in points.iter().skip(1) {
        path.line_to(Point::new(rig.canvas.x(point.0)?, rig.canvas.y(point.1)?))?;
    }
    let path_index = scene.push_path(path.finish(true)?)?;
    scene.push(Command::Path {
        path_index,
        fill_rule: FillRule::NonZero,
        fill: Some(Paint::solid(color)),
        stroke: None,
    })
}

pub(super) fn eyes(
    scene: &mut Scene,
    rig: FamilyRig,
    y: i32,
    spacing: i32,
    size: i32,
) -> Result<(), AvatarError> {
    for x in [50 - spacing, 50 + spacing] {
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(y)?),
            rig.canvas.s(size)?,
            rig.canvas.s(size + 1)?,
            rig.light,
        )?;
        ellipse(
            scene,
            Point::new(rig.canvas.x(x)?, rig.canvas.y(y)?),
            rig.canvas.s((size / 2).max(1))?,
            rig.canvas.s((size / 2).max(1))?,
            rig.ink,
        )?;
    }
    Ok(())
}

pub(super) fn smile(scene: &mut Scene, rig: FamilyRig, y: i32) -> Result<(), AvatarError> {
    let mut path = Path::builder(Point::new(rig.canvas.x(43)?, rig.canvas.y(y)?))?;
    path.quad_to(
        Point::new(rig.canvas.x(50)?, rig.canvas.y(y + 7)?),
        Point::new(rig.canvas.x(57)?, rig.canvas.y(y)?),
    )?;
    let path_index = scene.push_path(path.finish(false)?)?;
    scene.push(Command::Path {
        path_index,
        fill_rule: FillRule::NonZero,
        fill: None,
        stroke: Some(Stroke {
            width: rig.canvas.s(1)?,
            paint: Paint::solid(rig.ink),
        }),
    })
}
