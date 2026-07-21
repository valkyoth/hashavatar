use super::*;

pub fn encode_avatar<R: AvatarRenderer>(
    renderer: &R,
    spec: AvatarSpec,
    format: AvatarOutputFormat,
) -> ImageResult<Vec<u8>> {
    validate_image_avatar_spec(spec)?;
    let image = renderer
        .render(spec)
        .map_err(avatar_spec_error_to_image_error)?;
    let image = SanitizingRgbaImage::new(image);
    validate_renderer_output(spec, image.as_image())?;
    encode_rgba_image(image.as_image(), format)
}

fn validate_renderer_output(spec: AvatarSpec, image: &RgbaImage) -> ImageResult<()> {
    if image.dimensions() != (spec.width(), spec.height())
        || image.as_raw().len() != spec.rgba_buffer_len()
        || image.as_raw().len() > MAX_AVATAR_RGBA_BYTES
    {
        return Err(ImageError::Limits(LimitError::from_kind(
            LimitErrorKind::DimensionError,
        )));
    }

    Ok(())
}

/// Render and encode a cat avatar into memory.
pub fn encode_cat_avatar(spec: AvatarSpec, format: AvatarOutputFormat) -> ImageResult<Vec<u8>> {
    encode_avatar(&CatAvatar, spec, format)
}

/// Render and encode a cat avatar for a stable identity string.
pub fn encode_cat_avatar_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    format: AvatarOutputFormat,
) -> ImageResult<Vec<u8>> {
    let renderer = HashedCatAvatar::new(id).map_err(avatar_identity_error_to_image_error)?;
    encode_avatar(&renderer, spec, format)
}

pub fn encode_avatar_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    format: AvatarOutputFormat,
    options: AvatarOptions,
) -> ImageResult<Vec<u8>> {
    encode_avatar_for_namespace(spec, AvatarNamespace::default(), id, format, options)
}

pub fn encode_avatar_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    format: AvatarOutputFormat,
    options: AvatarOptions,
) -> ImageResult<Vec<u8>> {
    encode_avatar_with_identity_options(
        spec,
        AvatarIdentityOptions::new(namespace),
        id,
        format,
        options,
    )
}

pub fn encode_avatar_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
    format: AvatarOutputFormat,
    options: AvatarOptions,
) -> ImageResult<Vec<u8>> {
    let image = render_avatar_with_identity_options(spec, identity_options, id, options)
        .map_err(avatar_render_error_to_image_error)?;
    encode_owned_rgba_image(image, format)
}

pub fn encode_avatar_style_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    format: AvatarOutputFormat,
    style: AvatarStyleOptions,
) -> ImageResult<Vec<u8>> {
    encode_avatar_style_for_namespace(spec, AvatarNamespace::default(), id, format, style)
}

pub fn encode_avatar_style_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    format: AvatarOutputFormat,
    style: AvatarStyleOptions,
) -> ImageResult<Vec<u8>> {
    encode_avatar_style_with_identity_options(
        spec,
        AvatarIdentityOptions::new(namespace),
        id,
        format,
        style,
    )
}

pub fn encode_avatar_style_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
    format: AvatarOutputFormat,
    style: AvatarStyleOptions,
) -> ImageResult<Vec<u8>> {
    let image = render_avatar_style_with_identity_options(spec, identity_options, id, style)
        .map_err(avatar_render_error_to_image_error)?;
    encode_owned_rgba_image(image, format)
}

pub fn encode_avatar_auto_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    format: AvatarOutputFormat,
) -> ImageResult<Vec<u8>> {
    encode_avatar_auto_for_namespace(spec, AvatarNamespace::default(), id, format)
}

pub fn encode_avatar_auto_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    format: AvatarOutputFormat,
) -> ImageResult<Vec<u8>> {
    encode_avatar_auto_with_identity_options(
        spec,
        AvatarIdentityOptions::new(namespace),
        id,
        format,
    )
}

pub fn encode_avatar_auto_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
    format: AvatarOutputFormat,
) -> ImageResult<Vec<u8>> {
    let image = render_avatar_auto_with_identity_options(spec, identity_options, id)
        .map_err(avatar_render_error_to_image_error)?;
    encode_owned_rgba_image(image, format)
}

#[derive(Clone, Debug)]
pub(crate) struct AvatarRenderPlan {
    spec: AvatarSpec,
    identity: AvatarIdentity,
    style: AvatarStyleOptions,
}

impl AvatarRenderPlan {
    pub(crate) fn from_identity(
        spec: AvatarSpec,
        identity: AvatarIdentity,
        style: AvatarStyleOptions,
    ) -> Result<Self, AvatarSpecError> {
        spec.validate()?;
        Ok(Self {
            spec,
            identity,
            style,
        })
    }

    pub(crate) fn new<T: AsRef<[u8]>>(
        spec: AvatarSpec,
        identity_options: AvatarIdentityOptions<'_>,
        id: T,
        options: AvatarOptions,
    ) -> Result<Self, AvatarRenderError> {
        Self::new_with_style(
            spec,
            identity_options,
            id,
            AvatarStyleOptions::from(options),
        )
    }

    pub(crate) fn new_with_style<T: AsRef<[u8]>>(
        spec: AvatarSpec,
        identity_options: AvatarIdentityOptions<'_>,
        id: T,
        style: AvatarStyleOptions,
    ) -> Result<Self, AvatarRenderError> {
        spec.validate()?;
        let identity = AvatarIdentity::new_with_options(identity_options, id)?;
        Ok(Self {
            spec,
            identity,
            style,
        })
    }

    pub(crate) fn new_auto<T: AsRef<[u8]>>(
        spec: AvatarSpec,
        identity_options: AvatarIdentityOptions<'_>,
        id: T,
    ) -> Result<Self, AvatarRenderError> {
        spec.validate()?;
        let identity = AvatarIdentity::new_with_options(identity_options, id)?;
        let style = AvatarStyleOptions::from_identity(&identity);
        Ok(Self {
            spec,
            identity,
            style,
        })
    }

    pub(crate) fn validate_style_strict(&self) -> Result<(), AvatarStyleValidationError> {
        self.style.validate_strict()
    }

    pub(crate) const fn spec(&self) -> AvatarSpec {
        self.spec
    }

    pub(crate) const fn style(&self) -> AvatarStyleOptions {
        self.style
    }

    pub(crate) fn set_style(&mut self, style: AvatarStyleOptions) {
        self.style = style;
    }

    pub(crate) fn identity_cache_key(&self) -> IdentityCacheKey {
        self.identity.identity_cache_key()
    }

    pub(crate) fn avatar_asset_key(&self) -> AvatarAssetKey {
        self.identity.avatar_asset_key(self.spec, self.style)
    }

    pub(crate) fn encoded_asset_key(&self, format: AvatarOutputFormat) -> SemanticEncodedAssetKey {
        self.avatar_asset_key().encoded(format)
    }

    pub(crate) fn encoded_asset_key_for_build(
        &self,
        format: AvatarOutputFormat,
        build_id: EncoderBuildId,
    ) -> BuildEncodedAssetKey {
        self.avatar_asset_key().encoded_for_build(format, build_id)
    }

    pub(crate) fn render_rgba(&self) -> Result<RgbaImage, AvatarSpecError> {
        let mut image = match self.style.kind {
            AvatarKind::Cat => render_cat_avatar_for_identity_with_background(
                self.spec,
                &self.identity,
                self.style.background,
            ),
            AvatarKind::Dog => {
                render_dog_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Robot => {
                render_robot_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Fox => {
                render_fox_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Alien => {
                render_alien_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Monster => {
                render_monster_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Ghost => {
                render_ghost_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Slime => {
                render_slime_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Bird => {
                render_bird_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Wizard => {
                render_wizard_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Skull => {
                render_skull_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Paws => {
                render_paws_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Planet => {
                render_planet_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Rocket => {
                render_rocket_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Mushroom => render_mushroom_avatar_for_identity(
                self.spec,
                &self.identity,
                self.style.background,
            ),
            AvatarKind::Cactus => {
                render_cactus_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Frog => {
                render_frog_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Panda => {
                render_panda_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Cupcake => {
                render_cupcake_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Pizza => {
                render_pizza_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Icecream => render_icecream_avatar_for_identity(
                self.spec,
                &self.identity,
                self.style.background,
            ),
            AvatarKind::Octopus => {
                render_octopus_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Knight => {
                render_knight_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Bear => {
                render_bear_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Penguin => {
                render_penguin_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Dragon => {
                render_dragon_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Ninja => {
                render_ninja_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::Astronaut => render_astronaut_avatar_for_identity(
                self.spec,
                &self.identity,
                self.style.background,
            ),
            AvatarKind::Diamond => {
                render_diamond_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
            AvatarKind::CoffeeCup => render_coffee_cup_avatar_for_identity(
                self.spec,
                &self.identity,
                self.style.background,
            ),
            AvatarKind::Shield => {
                render_shield_avatar_for_identity(self.spec, &self.identity, self.style.background)
            }
        }?;

        apply_style_layers(&mut image, self.spec, self.style, &self.identity);
        Ok(image)
    }

    pub(crate) fn svg_background_color(&self) -> Color {
        match self.style.background {
            AvatarBackground::Themed => match self.style.kind {
                AvatarKind::Cat => {
                    hsl_to_color(28.0 + self.identity.unit_f32(2) * 40.0, 0.25, 0.92)
                }
                AvatarKind::Dog => {
                    hsl_to_color(200.0 + self.identity.unit_f32(3) * 60.0, 0.20, 0.92)
                }
                AvatarKind::Robot => {
                    hsl_to_color(220.0 + self.identity.unit_f32(4) * 50.0, 0.18, 0.93)
                }
                AvatarKind::Fox => {
                    hsl_to_color(18.0 + self.identity.unit_f32(5) * 30.0, 0.28, 0.93)
                }
                AvatarKind::Alien => {
                    hsl_to_color(260.0 + self.identity.unit_f32(6) * 60.0, 0.20, 0.93)
                }
                AvatarKind::Monster => {
                    hsl_to_color(300.0 + self.identity.unit_f32(7) * 45.0, 0.24, 0.92)
                }
                AvatarKind::Ghost => {
                    hsl_to_color(220.0 + self.identity.unit_f32(8) * 35.0, 0.18, 0.95)
                }
                AvatarKind::Slime => {
                    hsl_to_color(120.0 + self.identity.unit_f32(9) * 70.0, 0.24, 0.92)
                }
                AvatarKind::Bird => {
                    hsl_to_color(180.0 + self.identity.unit_f32(10) * 40.0, 0.22, 0.93)
                }
                AvatarKind::Wizard => {
                    hsl_to_color(250.0 + self.identity.unit_f32(11) * 40.0, 0.24, 0.92)
                }
                AvatarKind::Skull => {
                    hsl_to_color(210.0 + self.identity.unit_f32(12) * 20.0, 0.08, 0.94)
                }
                AvatarKind::Paws => {
                    hsl_to_color(28.0 + self.identity.unit_f32(13) * 30.0, 0.22, 0.94)
                }
                AvatarKind::Planet => {
                    hsl_to_color(215.0 + self.identity.unit_f32(14) * 90.0, 0.24, 0.91)
                }
                AvatarKind::Rocket => {
                    hsl_to_color(205.0 + self.identity.unit_f32(15) * 70.0, 0.22, 0.92)
                }
                AvatarKind::Mushroom => {
                    hsl_to_color(18.0 + self.identity.unit_f32(16) * 35.0, 0.20, 0.93)
                }
                AvatarKind::Cactus => {
                    hsl_to_color(80.0 + self.identity.unit_f32(17) * 55.0, 0.20, 0.92)
                }
                AvatarKind::Frog => {
                    hsl_to_color(95.0 + self.identity.unit_f32(18) * 65.0, 0.23, 0.92)
                }
                AvatarKind::Panda => {
                    hsl_to_color(200.0 + self.identity.unit_f32(19) * 45.0, 0.08, 0.94)
                }
                AvatarKind::Cupcake => {
                    hsl_to_color(320.0 + self.identity.unit_f32(20) * 45.0, 0.22, 0.94)
                }
                AvatarKind::Pizza => {
                    hsl_to_color(36.0 + self.identity.unit_f32(21) * 30.0, 0.24, 0.93)
                }
                AvatarKind::Icecream => {
                    hsl_to_color(190.0 + self.identity.unit_f32(22) * 95.0, 0.18, 0.94)
                }
                AvatarKind::Octopus => {
                    hsl_to_color(185.0 + self.identity.unit_f32(23) * 70.0, 0.22, 0.92)
                }
                AvatarKind::Knight => {
                    hsl_to_color(215.0 + self.identity.unit_f32(24) * 30.0, 0.12, 0.92)
                }
                AvatarKind::Bear => {
                    hsl_to_color(30.0 + self.identity.unit_f32(25) * 28.0, 0.20, 0.92)
                }
                AvatarKind::Penguin => {
                    hsl_to_color(200.0 + self.identity.unit_f32(26) * 35.0, 0.18, 0.93)
                }
                AvatarKind::Dragon => {
                    hsl_to_color(105.0 + self.identity.unit_f32(27) * 45.0, 0.22, 0.91)
                }
                AvatarKind::Ninja => {
                    hsl_to_color(225.0 + self.identity.unit_f32(28) * 35.0, 0.12, 0.91)
                }
                AvatarKind::Astronaut => {
                    hsl_to_color(215.0 + self.identity.unit_f32(29) * 60.0, 0.16, 0.92)
                }
                AvatarKind::Diamond => {
                    hsl_to_color(185.0 + self.identity.unit_f32(30) * 50.0, 0.20, 0.93)
                }
                AvatarKind::CoffeeCup => {
                    hsl_to_color(32.0 + self.identity.unit_f32(31) * 28.0, 0.18, 0.93)
                }
                AvatarKind::Shield => {
                    hsl_to_color(215.0 + self.identity.unit_f32(32) * 40.0, 0.15, 0.92)
                }
            },
            AvatarBackground::White => Color::rgb(255, 255, 255),
            AvatarBackground::Black => Color::rgb(0, 0, 0),
            AvatarBackground::Dark => Color::rgb(17, 24, 39),
            AvatarBackground::Light => Color::rgb(248, 250, 247),
            AvatarBackground::Transparent => Color::rgba(255, 255, 255, 0),
            AvatarBackground::PolkaDot
            | AvatarBackground::Striped
            | AvatarBackground::Checkerboard
            | AvatarBackground::Grid => Color::rgb(248, 250, 247),
            AvatarBackground::Sunrise => Color::rgb(255, 247, 212),
            AvatarBackground::Ocean => Color::rgb(220, 248, 252),
            AvatarBackground::Starry => Color::rgb(17, 24, 39),
        }
    }

    pub(crate) fn render_svg_body(&self) -> String {
        match self.style.kind {
            AvatarKind::Cat => render_cat_svg(self.spec, &self.identity),
            AvatarKind::Dog => render_dog_svg(self.spec, &self.identity),
            AvatarKind::Robot => render_robot_svg(self.spec, &self.identity),
            AvatarKind::Fox => render_fox_svg(self.spec, &self.identity),
            AvatarKind::Alien => render_alien_svg(self.spec, &self.identity),
            AvatarKind::Monster => render_monster_svg(self.spec, &self.identity),
            AvatarKind::Ghost => render_ghost_svg(self.spec, &self.identity),
            AvatarKind::Slime => render_slime_svg(self.spec, &self.identity),
            AvatarKind::Bird => render_bird_svg(self.spec, &self.identity),
            AvatarKind::Wizard => render_wizard_svg(self.spec, &self.identity),
            AvatarKind::Skull => render_skull_svg(self.spec, &self.identity),
            AvatarKind::Paws => render_paws_svg(self.spec, &self.identity),
            AvatarKind::Planet => render_planet_svg(self.spec, &self.identity),
            AvatarKind::Rocket => render_rocket_svg(self.spec, &self.identity),
            AvatarKind::Mushroom => render_mushroom_svg(self.spec, &self.identity),
            AvatarKind::Cactus => render_cactus_svg(self.spec, &self.identity),
            AvatarKind::Frog => render_frog_svg(self.spec, &self.identity),
            AvatarKind::Panda => render_panda_svg(self.spec, &self.identity),
            AvatarKind::Cupcake => render_cupcake_svg(self.spec, &self.identity),
            AvatarKind::Pizza => render_pizza_svg(self.spec, &self.identity),
            AvatarKind::Icecream => render_icecream_svg(self.spec, &self.identity),
            AvatarKind::Octopus => render_octopus_svg(self.spec, &self.identity),
            AvatarKind::Knight => render_knight_svg(self.spec, &self.identity),
            AvatarKind::Bear => render_bear_svg(self.spec, &self.identity),
            AvatarKind::Penguin => render_penguin_svg(self.spec, &self.identity),
            AvatarKind::Dragon => render_dragon_svg(self.spec, &self.identity),
            AvatarKind::Ninja => render_ninja_svg(self.spec, &self.identity),
            AvatarKind::Astronaut => render_astronaut_svg(self.spec, &self.identity),
            AvatarKind::Diamond => render_diamond_svg(self.spec, &self.identity),
            AvatarKind::CoffeeCup => render_coffee_cup_svg(self.spec, &self.identity),
            AvatarKind::Shield => render_shield_svg(self.spec, &self.identity),
        }
    }

    pub(crate) fn render_svg(&self) -> String {
        let definition_prefix = self.svg_definition_prefix();
        let background = self.render_svg_background(&definition_prefix);

        let content = format!(
            "{}{}{}",
            background,
            self.render_svg_body(),
            render_style_svg_layers(self.spec, self.style, &self.identity)
        );
        let (clip_defs, clipped_content) =
            render_shape_svg_clip(self.spec, self.style.shape, &definition_prefix, &content);
        let shape_layer = render_shape_svg_layer(
            self.spec,
            self.style.shape,
            style_accent_color(self.style.color, &self.identity),
        );

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" role="img" aria-label="{label} avatar">{clip_defs}{body}{shape_layer}</svg>"#,
            w = self.spec.width,
            h = self.spec.height,
            label = self.style.kind.as_str(),
            clip_defs = clip_defs,
            body = clipped_content,
            shape_layer = shape_layer,
        )
    }

    fn svg_definition_prefix(&self) -> String {
        format!(
            "hashavatar-{}x{}-{}-{}",
            self.spec.width,
            self.spec.height,
            self.style.shape.as_str(),
            self.style.background.as_str(),
        )
    }

    fn render_svg_background(&self, definition_prefix: &str) -> String {
        match self.style.background {
            AvatarBackground::Transparent => String::new(),
            AvatarBackground::Themed
            | AvatarBackground::White
            | AvatarBackground::Black
            | AvatarBackground::Dark
            | AvatarBackground::Light => {
                format!(
                    r#"<rect width="100%" height="100%" fill="{}"/>"#,
                    color_hex(self.svg_background_color())
                )
            }
            AvatarBackground::PolkaDot => {
                let id = format!("{definition_prefix}-bg-polka-dot");
                format!(
                    r##"<defs><pattern id="{id}" width="16" height="16" patternUnits="userSpaceOnUse"><rect width="16" height="16" fill="#f8faf7"/><circle cx="8" cy="8" r="2" fill="#d1d5db"/></pattern></defs><rect width="100%" height="100%" fill="url(#{id})"/>"##
                )
            }
            AvatarBackground::Striped => {
                let id = format!("{definition_prefix}-bg-striped");
                format!(
                    r##"<defs><pattern id="{id}" width="18" height="18" patternUnits="userSpaceOnUse" patternTransform="rotate(45)"><rect width="18" height="18" fill="#f8faf7"/><rect width="9" height="18" fill="#e5e7eb"/></pattern></defs><rect width="100%" height="100%" fill="url(#{id})"/>"##
                )
            }
            AvatarBackground::Checkerboard => {
                let id = format!("{definition_prefix}-bg-checkerboard");
                format!(
                    r##"<defs><pattern id="{id}" width="24" height="24" patternUnits="userSpaceOnUse"><rect width="24" height="24" fill="#f8faf7"/><rect width="12" height="12" fill="#e8ece7"/><rect x="12" y="12" width="12" height="12" fill="#e8ece7"/></pattern></defs><rect width="100%" height="100%" fill="url(#{id})"/>"##
                )
            }
            AvatarBackground::Grid => {
                let id = format!("{definition_prefix}-bg-grid");
                format!(
                    r##"<defs><pattern id="{id}" width="16" height="16" patternUnits="userSpaceOnUse"><rect width="16" height="16" fill="#f8faf7"/><path d="M 16 0 L 0 0 0 16" fill="none" stroke="#dde2dd" stroke-width="1"/></pattern></defs><rect width="100%" height="100%" fill="url(#{id})"/>"##
                )
            }
            AvatarBackground::Sunrise => {
                let id = format!("{definition_prefix}-bg-sunrise");
                format!(
                    r##"<defs><linearGradient id="{id}" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="#fff7d4"/><stop offset="100%" stop-color="#ffb86b"/></linearGradient></defs><rect width="100%" height="100%" fill="url(#{id})"/>"##
                )
            }
            AvatarBackground::Ocean => {
                let id = format!("{definition_prefix}-bg-ocean");
                format!(
                    r##"<defs><linearGradient id="{id}" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="#dcf8fc"/><stop offset="100%" stop-color="#4b91be"/></linearGradient></defs><rect width="100%" height="100%" fill="url(#{id})"/>"##
                )
            }
            AvatarBackground::Starry => {
                let id = format!("{definition_prefix}-bg-starry");
                format!(
                    r##"<defs><pattern id="{id}" width="40" height="40" patternUnits="userSpaceOnUse"><rect width="40" height="40" fill="#111827"/><circle cx="8" cy="9" r="1.2" fill="#ffffff" opacity="0.7"/><circle cx="28" cy="14" r="1" fill="#ffffff" opacity="0.55"/><circle cx="18" cy="31" r="1.4" fill="#ffffff" opacity="0.65"/></pattern></defs><rect width="100%" height="100%" fill="url(#{id})"/>"##
                )
            }
        }
    }
}

/// Fluent high-level API for common avatar rendering paths.
///
/// The builder is additive over the lower-level free functions. It stores
/// validation failures and returns them from render/encode methods, so invalid
/// size or namespace input remains non-panicking.
#[derive(Clone)]
pub struct AvatarBuilder<'a, T> {
    id: T,
    spec: Result<AvatarSpec, AvatarSpecError>,
    namespace: Result<AvatarNamespace<'a>, AvatarIdentityError>,
    style: Option<AvatarStyleOptions>,
}

impl<T> std::fmt::Debug for AvatarBuilder<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let namespace = self.namespace.as_ref().map(|_| "[REDACTED]");
        f.debug_struct("AvatarBuilder")
            .field("id", &"[REDACTED]")
            .field("spec", &self.spec)
            .field("namespace", &namespace)
            .field("style", &self.style)
            .finish()
    }
}

impl<T> AvatarBuilder<'static, T> {
    /// Starts a builder for an identity input.
    pub fn for_id(id: T) -> Self {
        Self {
            id,
            spec: Ok(AvatarSpec::default()),
            namespace: Ok(AvatarNamespace::default()),
            style: Some(AvatarStyleOptions::default()),
        }
    }
}

impl<'a, T> AvatarBuilder<'a, T> {
    /// Uses an already validated avatar specification.
    pub fn spec(mut self, spec: AvatarSpec) -> Self {
        self.spec = Ok(spec);
        self
    }

    /// Sets the avatar dimensions while preserving the current style variant
    /// seed when possible.
    pub fn size(mut self, width: u32, height: u32) -> Self {
        let seed = self.spec.ok().map(AvatarSpec::seed).unwrap_or_default();
        self.spec = AvatarSpec::new(width, height, seed);
        self
    }

    /// Sets the caller-controlled style variant seed.
    ///
    /// The seed is mixed into the identity-derived renderer RNG. It is useful
    /// when an application wants a second deterministic variant for the same
    /// identity without changing tenant or namespace style-version values.
    pub fn style_variant(mut self, seed: u64) -> Self {
        self.spec = match self.spec {
            Ok(spec) => Ok(AvatarSpec::new_unchecked(spec.width, spec.height, seed)),
            Err(error) => Err(error),
        };
        self
    }

    /// Alias for [`AvatarBuilder::style_variant`].
    pub fn seed(self, seed: u64) -> Self {
        self.style_variant(seed)
    }

    /// Sets the identity namespace used for tenant isolation and visual
    /// rollout control.
    pub fn namespace<'b>(self, tenant: &'b str, style_version: &'b str) -> AvatarBuilder<'b, T> {
        AvatarBuilder {
            id: self.id,
            spec: self.spec,
            namespace: AvatarNamespace::new(tenant, style_version),
            style: self.style,
        }
    }

    /// Uses automatic style derivation from distinct identity digest bytes.
    pub fn automatic_style(mut self) -> Self {
        self.style = None;
        self
    }

    pub fn style(mut self, style: AvatarStyleOptions) -> Self {
        self.style = Some(style);
        self
    }

    pub fn options(self, options: AvatarOptions) -> Self {
        self.style(AvatarStyleOptions::from_options(options))
    }

    pub fn kind(self, kind: AvatarKind) -> Self {
        self.with_style(|style| style.kind = kind)
    }

    pub fn background(self, background: AvatarBackground) -> Self {
        self.with_style(|style| style.background = background)
    }

    pub fn accessory(self, accessory: AvatarAccessory) -> Self {
        self.with_style(|style| style.accessory = accessory)
    }

    pub fn color(self, color: AvatarColor) -> Self {
        self.with_style(|style| style.color = color)
    }

    pub fn expression(self, expression: AvatarExpression) -> Self {
        self.with_style(|style| style.expression = expression)
    }

    pub fn shape(self, shape: AvatarShape) -> Self {
        self.with_style(|style| style.shape = shape)
    }

    fn with_style(mut self, update: impl FnOnce(&mut AvatarStyleOptions)) -> Self {
        let style = self.style.get_or_insert_with(AvatarStyleOptions::default);
        update(style);
        self
    }
}

impl<'a, T: AsRef<[u8]>> AvatarBuilder<'a, T> {
    pub fn identity(&self) -> Result<AvatarIdentity, AvatarError> {
        let namespace = self.namespace?;
        Ok(AvatarIdentity::new_with_namespace(
            namespace,
            self.id.as_ref(),
        )?)
    }

    pub fn cache_key(&self) -> Result<String, AvatarError> {
        Ok(self.identity()?.cache_key())
    }

    /// Returns a typed, domain-separated identity cache key.
    pub fn identity_cache_key(&self) -> Result<IdentityCacheKey, AvatarError> {
        Ok(self.identity()?.identity_cache_key())
    }

    /// Returns the complete cache key for this resolved unencoded avatar.
    pub fn avatar_asset_key(&self) -> Result<AvatarAssetKey, AvatarError> {
        Ok(self.render_plan()?.avatar_asset_key())
    }

    /// Returns the semantic cache key for this resolved encoded avatar.
    pub fn encoded_asset_key(
        &self,
        format: AvatarOutputFormat,
    ) -> Result<SemanticEncodedAssetKey, AvatarError> {
        Ok(self.render_plan()?.encoded_asset_key(format))
    }

    /// Returns a deployment-specific key for this resolved encoded avatar.
    pub fn encoded_asset_key_for_build(
        &self,
        format: AvatarOutputFormat,
        build_id: EncoderBuildId,
    ) -> Result<BuildEncodedAssetKey, AvatarError> {
        Ok(self
            .render_plan()?
            .encoded_asset_key_for_build(format, build_id))
    }

    /// Switches to an opt-in builder that rejects unsupported style layers.
    ///
    /// Configure the ordinary builder first, then call this method before
    /// rendering, encoding, or deriving a complete asset key.
    pub fn strict_style_validation(self) -> StrictAvatarBuilder<'a, T> {
        StrictAvatarBuilder { inner: self }
    }

    pub fn render(self) -> Result<RgbaImage, AvatarError> {
        self.render_plan()?.render_rgba().map_err(AvatarError::from)
    }

    pub fn render_svg(self) -> Result<String, AvatarError> {
        Ok(self.render_plan()?.render_svg())
    }

    pub fn encode(self, format: AvatarOutputFormat) -> Result<Vec<u8>, AvatarError> {
        let image = self.render_plan()?.render_rgba()?;
        Ok(encode_owned_rgba_image(image, format)?)
    }

    /// Freezes this legacy builder into the 1.3 prepared-request preview.
    ///
    /// This preserves the builder's existing skip-on-unsupported behavior.
    /// Use [`AvatarRequestBuilder`] for strict-by-default new integrations.
    pub fn prepare(self) -> Result<PreparedAvatar, AvatarError> {
        let automatically_derived = self.style.is_none();
        Ok(PreparedAvatar::from_legacy_plan(
            self.render_plan()?,
            automatically_derived,
        ))
    }

    fn render_plan(&self) -> Result<AvatarRenderPlan, AvatarError> {
        let spec = self.spec?;
        let namespace = self.namespace?;
        let identity_options = AvatarIdentityOptions::new(namespace);
        match self.style {
            Some(style) => Ok(AvatarRenderPlan::new_with_style(
                spec,
                identity_options,
                self.id.as_ref(),
                style,
            )?),
            None => Ok(AvatarRenderPlan::new_auto(
                spec,
                identity_options,
                self.id.as_ref(),
            )?),
        }
    }
}

/// Error returned by opt-in strict builder operations.
#[non_exhaustive]
#[derive(Debug)]
pub enum StrictAvatarError {
    Avatar(AvatarError),
    Style(AvatarStyleValidationError),
}

impl From<AvatarError> for StrictAvatarError {
    fn from(error: AvatarError) -> Self {
        Self::Avatar(error)
    }
}

impl From<AvatarStyleValidationError> for StrictAvatarError {
    fn from(error: AvatarStyleValidationError) -> Self {
        Self::Style(error)
    }
}

impl std::fmt::Display for StrictAvatarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Avatar(error) => error.fmt(f),
            Self::Style(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for StrictAvatarError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Avatar(error) => Some(error),
            Self::Style(error) => Some(error),
        }
    }
}

/// Opt-in high-level API that rejects unsupported explicit style layers.
///
/// Legacy rendering intentionally skips accessories and expressions for
/// families without face anchors. This wrapper changes only validation: a
/// style that would be skipped is returned as [`StrictAvatarError::Style`]
/// before rendering or encoding starts.
#[derive(Clone)]
pub struct StrictAvatarBuilder<'a, T> {
    inner: AvatarBuilder<'a, T>,
}

impl<T> std::fmt::Debug for StrictAvatarBuilder<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StrictAvatarBuilder")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<'a, T> StrictAvatarBuilder<'a, T> {
    /// Returns to the legacy skip-on-unsupported builder behavior.
    pub fn into_legacy_builder(self) -> AvatarBuilder<'a, T> {
        self.inner
    }
}

impl<'a, T: AsRef<[u8]>> StrictAvatarBuilder<'a, T> {
    fn validated_plan(&self) -> Result<AvatarRenderPlan, StrictAvatarError> {
        let plan = self.inner.render_plan()?;
        plan.validate_style_strict()?;
        Ok(plan)
    }

    pub fn avatar_asset_key(&self) -> Result<AvatarAssetKey, StrictAvatarError> {
        Ok(self.validated_plan()?.avatar_asset_key())
    }

    pub fn encoded_asset_key(
        &self,
        format: AvatarOutputFormat,
    ) -> Result<SemanticEncodedAssetKey, StrictAvatarError> {
        Ok(self.validated_plan()?.encoded_asset_key(format))
    }

    /// Returns a deployment-specific key after strict style validation.
    pub fn encoded_asset_key_for_build(
        &self,
        format: AvatarOutputFormat,
        build_id: EncoderBuildId,
    ) -> Result<BuildEncodedAssetKey, StrictAvatarError> {
        Ok(self
            .validated_plan()?
            .encoded_asset_key_for_build(format, build_id))
    }

    pub fn render(self) -> Result<RgbaImage, StrictAvatarError> {
        self.validated_plan()?
            .render_rgba()
            .map_err(AvatarError::from)
            .map_err(StrictAvatarError::from)
    }

    pub fn render_svg(self) -> Result<String, StrictAvatarError> {
        Ok(self.validated_plan()?.render_svg())
    }

    pub fn encode(self, format: AvatarOutputFormat) -> Result<Vec<u8>, StrictAvatarError> {
        let image = self
            .validated_plan()?
            .render_rgba()
            .map_err(AvatarError::from)?;
        encode_owned_rgba_image(image, format)
            .map_err(AvatarError::from)
            .map_err(StrictAvatarError::from)
    }

    /// Freezes this strictly validated builder into a prepared request.
    pub fn prepare(self) -> Result<PreparedAvatar, StrictAvatarError> {
        let automatically_derived = self.inner.style.is_none();
        Ok(PreparedAvatar::from_legacy_plan(
            self.validated_plan()?,
            automatically_derived,
        ))
    }
}

/// Render an avatar image directly without encoding it.
pub fn render_avatar_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    options: AvatarOptions,
) -> Result<RgbaImage, AvatarRenderError> {
    render_avatar_for_namespace(spec, AvatarNamespace::default(), id, options)
}

pub fn render_avatar_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    options: AvatarOptions,
) -> Result<RgbaImage, AvatarRenderError> {
    render_avatar_with_identity_options(spec, AvatarIdentityOptions::new(namespace), id, options)
}

pub fn render_avatar_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
    options: AvatarOptions,
) -> Result<RgbaImage, AvatarRenderError> {
    AvatarRenderPlan::new(spec, identity_options, id, options)?
        .render_rgba()
        .map_err(AvatarRenderError::from)
}

/// Render an avatar image with explicit visual layer style options.
pub fn render_avatar_style_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    style: AvatarStyleOptions,
) -> Result<RgbaImage, AvatarRenderError> {
    render_avatar_style_for_namespace(spec, AvatarNamespace::default(), id, style)
}

pub fn render_avatar_style_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    style: AvatarStyleOptions,
) -> Result<RgbaImage, AvatarRenderError> {
    render_avatar_style_with_identity_options(
        spec,
        AvatarIdentityOptions::new(namespace),
        id,
        style,
    )
}

pub fn render_avatar_style_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
    style: AvatarStyleOptions,
) -> Result<RgbaImage, AvatarRenderError> {
    AvatarRenderPlan::new_with_style(spec, identity_options, id, style)?
        .render_rgba()
        .map_err(AvatarRenderError::from)
}

/// Render an avatar image with all public style choices derived from distinct
/// identity digest bytes.
pub fn render_avatar_auto_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
) -> Result<RgbaImage, AvatarRenderError> {
    render_avatar_auto_for_namespace(spec, AvatarNamespace::default(), id)
}

pub fn render_avatar_auto_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
) -> Result<RgbaImage, AvatarRenderError> {
    render_avatar_auto_with_identity_options(spec, AvatarIdentityOptions::new(namespace), id)
}

pub fn render_avatar_auto_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
) -> Result<RgbaImage, AvatarRenderError> {
    AvatarRenderPlan::new_auto(spec, identity_options, id)?
        .render_rgba()
        .map_err(AvatarRenderError::from)
}

/// Render an avatar as a compact SVG string.
pub fn render_avatar_svg_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    options: AvatarOptions,
) -> Result<String, AvatarRenderError> {
    render_avatar_svg_for_namespace(spec, AvatarNamespace::default(), id, options)
}

pub fn render_avatar_svg_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    options: AvatarOptions,
) -> Result<String, AvatarRenderError> {
    render_avatar_svg_with_identity_options(
        spec,
        AvatarIdentityOptions::new(namespace),
        id,
        options,
    )
}

pub fn render_avatar_svg_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
    options: AvatarOptions,
) -> Result<String, AvatarRenderError> {
    Ok(AvatarRenderPlan::new(spec, identity_options, id, options)?.render_svg())
}

/// Render an avatar with explicit visual layer style options as a compact SVG string.
pub fn render_avatar_svg_style_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
    style: AvatarStyleOptions,
) -> Result<String, AvatarRenderError> {
    render_avatar_svg_style_for_namespace(spec, AvatarNamespace::default(), id, style)
}

pub fn render_avatar_svg_style_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
    style: AvatarStyleOptions,
) -> Result<String, AvatarRenderError> {
    render_avatar_svg_style_with_identity_options(
        spec,
        AvatarIdentityOptions::new(namespace),
        id,
        style,
    )
}

pub fn render_avatar_svg_style_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
    style: AvatarStyleOptions,
) -> Result<String, AvatarRenderError> {
    Ok(AvatarRenderPlan::new_with_style(spec, identity_options, id, style)?.render_svg())
}

/// Render an avatar SVG with all public style choices derived from distinct
/// identity digest bytes.
pub fn render_avatar_svg_auto_for_id<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    id: T,
) -> Result<String, AvatarRenderError> {
    render_avatar_svg_auto_for_namespace(spec, AvatarNamespace::default(), id)
}

pub fn render_avatar_svg_auto_for_namespace<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    namespace: AvatarNamespace<'_>,
    id: T,
) -> Result<String, AvatarRenderError> {
    render_avatar_svg_auto_with_identity_options(spec, AvatarIdentityOptions::new(namespace), id)
}

pub fn render_avatar_svg_auto_with_identity_options<T: AsRef<[u8]>>(
    spec: AvatarSpec,
    identity_options: AvatarIdentityOptions<'_>,
    id: T,
) -> Result<String, AvatarRenderError> {
    Ok(AvatarRenderPlan::new_auto(spec, identity_options, id)?.render_svg())
}
