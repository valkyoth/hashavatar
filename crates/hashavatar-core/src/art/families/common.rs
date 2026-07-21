use crate::{
    AvatarTraitVector, CatError,
    art::util::{Canvas, themed_color, vary},
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
    pub(super) fn new(scene: &Scene, traits: AvatarTraitVector) -> Result<Self, CatError> {
        Ok(Self {
            canvas: Canvas::new(scene)?,
            traits,
            primary: themed_color(traits.primary_hue(), 72, 224, 5),
            secondary: themed_color(traits.secondary_hue(), 96, 238, 9),
            accent: themed_color(traits.accent_hue(), 74, 232, 12),
            light: Color::rgb(244, 247, 243),
            ink: Color::rgb(25, 29, 36),
        })
    }

    pub(super) fn head_rx(self) -> Result<Fixed, CatError> {
        vary(self.canvas.minimum, 24, 30, self.traits.proportion_a())
    }

    pub(super) fn head_ry(self) -> Result<Fixed, CatError> {
        vary(self.canvas.minimum, 23, 30, self.traits.proportion_b())
    }
}

pub(super) fn ellipse(
    scene: &mut Scene,
    center: Point,
    radius_x: Fixed,
    radius_y: Fixed,
    color: Color,
) -> Result<(), CatError> {
    scene.push(Command::Ellipse {
        center,
        radius_x,
        radius_y,
        paint: Paint::solid(color),
    })
}

pub(super) fn rect(scene: &mut Scene, rect: Rect, color: Color) -> Result<(), CatError> {
    scene.push(Command::Rect {
        rect,
        paint: Paint::solid(color),
    })
}

pub(super) fn triangle(
    scene: &mut Scene,
    points: [Point; 3],
    color: Color,
) -> Result<(), CatError> {
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
) -> Result<(), CatError> {
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
) -> Result<(), CatError> {
    let first = points.first().ok_or(CatError::InvalidScene)?;
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
) -> Result<(), CatError> {
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

pub(super) fn smile(scene: &mut Scene, rig: FamilyRig, y: i32) -> Result<(), CatError> {
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
