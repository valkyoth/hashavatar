use crate::{
    AvatarAnchorPoint, AvatarAnchorSet, AvatarColorRoles, AvatarError,
    art::util::{Canvas, role_color, scale},
    fixed::Fixed,
    geometry::{FillRule, Path, Point, Rect},
    paint::{Color, Paint},
    scene::{Command, Scene, Stroke},
};

#[derive(Clone, Copy)]
pub(super) struct LayerRig {
    pub(super) canvas: Canvas,
    pub(super) anchors: AvatarAnchorSet,
    pub(super) accent: Color,
    pub(super) light: Color,
    pub(super) ink: Color,
}

impl LayerRig {
    pub(super) fn new(
        scene: &Scene,
        anchors: AvatarAnchorSet,
        colors: AvatarColorRoles,
    ) -> Result<Self, AvatarError> {
        Ok(Self {
            canvas: Canvas::new(scene)?,
            anchors,
            accent: role_color(colors.accent()),
            light: role_color(colors.light()),
            ink: role_color(colors.ink()),
        })
    }

    pub(super) fn point(
        self,
        anchor: AvatarAnchorPoint,
        vertical_adjustment: i16,
    ) -> Result<Point, AvatarError> {
        Ok(Point::new(
            scale(
                self.canvas.width,
                i32::from(anchor.x_basis_points()),
                10_000,
            )?,
            scale(
                self.canvas.height,
                i32::from(anchor.y_basis_points()) + i32::from(vertical_adjustment),
                10_000,
            )?,
        ))
    }

    pub(super) fn size(self, basis_points: i32) -> Result<Fixed, AvatarError> {
        scale(self.canvas.minimum, basis_points, 10_000)
    }

    pub(super) fn face_half(self) -> Result<Fixed, AvatarError> {
        self.size(i32::from(self.anchors.face_width_basis_points()) / 2)
    }

    pub(super) fn eye_radius(self) -> Result<Fixed, AvatarError> {
        self.size(i32::from(self.anchors.eye_radius_basis_points()))
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

pub(super) fn rect(
    scene: &mut Scene,
    left: Fixed,
    top: Fixed,
    right: Fixed,
    bottom: Fixed,
    color: Color,
) -> Result<(), AvatarError> {
    scene.push(Command::Rect {
        rect: Rect::new(left, top, right, bottom),
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

pub(super) fn polygon(
    scene: &mut Scene,
    points: &[Point],
    color: Color,
) -> Result<(), AvatarError> {
    let first = *points.first().ok_or(AvatarError::InvalidScene)?;
    let mut path = Path::builder(first)?;
    for point in points.iter().copied().skip(1) {
        path.line_to(point)?;
    }
    let path_index = scene.push_path(path.finish(true)?)?;
    scene.push(Command::Path {
        path_index,
        fill_rule: FillRule::NonZero,
        fill: Some(Paint::solid(color)),
        stroke: None,
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

pub(super) fn curved_line(
    scene: &mut Scene,
    start: Point,
    control: Point,
    end: Point,
    width: Fixed,
    color: Color,
) -> Result<(), AvatarError> {
    let mut path = Path::builder(start)?;
    path.quad_to(control, end)?;
    let path_index = scene.push_path(path.finish(false)?)?;
    scene.push(Command::Path {
        path_index,
        fill_rule: FillRule::NonZero,
        fill: None,
        stroke: Some(Stroke {
            width,
            paint: Paint::solid(color),
        }),
    })
}

pub(super) fn outline_ellipse(
    scene: &mut Scene,
    center: Point,
    radius_x: Fixed,
    radius_y: Fixed,
    width: Fixed,
    color: Color,
) -> Result<(), AvatarError> {
    let kappa = Fixed::from_ratio(5_523, 10_000)?;
    let control_x = radius_x.checked_mul(kappa)?;
    let control_y = radius_y.checked_mul(kappa)?;
    let left = center.x.checked_sub(radius_x)?;
    let right = center.x.checked_add(radius_x)?;
    let top = center.y.checked_sub(radius_y)?;
    let bottom = center.y.checked_add(radius_y)?;
    let mut path = Path::builder(Point::new(left, center.y))?;
    path.cubic_to(
        Point::new(left, center.y.checked_sub(control_y)?),
        Point::new(center.x.checked_sub(control_x)?, top),
        Point::new(center.x, top),
    )?;
    path.cubic_to(
        Point::new(center.x.checked_add(control_x)?, top),
        Point::new(right, center.y.checked_sub(control_y)?),
        Point::new(right, center.y),
    )?;
    path.cubic_to(
        Point::new(right, center.y.checked_add(control_y)?),
        Point::new(center.x.checked_add(control_x)?, bottom),
        Point::new(center.x, bottom),
    )?;
    path.cubic_to(
        Point::new(center.x.checked_sub(control_x)?, bottom),
        Point::new(left, center.y.checked_add(control_y)?),
        Point::new(left, center.y),
    )?;
    let path_index = scene.push_path(path.finish(false)?)?;
    scene.push(Command::Path {
        path_index,
        fill_rule: FillRule::NonZero,
        fill: None,
        stroke: Some(Stroke {
            width,
            paint: Paint::solid(color),
        }),
    })
}
