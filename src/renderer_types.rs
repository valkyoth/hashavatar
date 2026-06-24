/// Cat-face avatar renderer built from simple geometric primitives.
///
/// The face is intentionally stylized:
/// - a rounded head ellipse defines the main silhouette
/// - two ear polygons make the head read as feline rather than circular
/// - wide-set eyes, a small triangular nose, whiskers, and a curved smile complete the expression
#[derive(Clone, Copy, Debug, Default)]
pub struct CatAvatar;

impl AvatarRenderer for CatAvatar {
    fn render(&self, spec: AvatarSpec) -> Result<RgbaImage, AvatarSpecError> {
        render_cat_avatar(spec)
    }
}

/// Cat-face avatar renderer driven by a stable identity digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashedCatAvatar {
    identity: AvatarIdentity,
}

impl HashedCatAvatar {
    pub fn new<T: AsRef<[u8]>>(input: T) -> Result<Self, AvatarIdentityError> {
        Self::new_with_namespace(AvatarNamespace::default(), input)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_namespace(namespace, input)?,
        })
    }

    pub fn new_with_identity_options<T: AsRef<[u8]>>(
        options: AvatarIdentityOptions<'_>,
        input: T,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_options(options, input)?,
        })
    }

    pub fn identity(&self) -> &AvatarIdentity {
        &self.identity
    }
}

impl AvatarRenderer for HashedCatAvatar {
    fn render(&self, spec: AvatarSpec) -> Result<RgbaImage, AvatarSpecError> {
        render_cat_avatar_for_identity(spec, &self.identity)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashedDogAvatar {
    identity: AvatarIdentity,
    background: AvatarBackground,
}

impl HashedDogAvatar {
    pub fn new<T: AsRef<[u8]>>(
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Self::new_with_namespace(AvatarNamespace::default(), input, background)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_namespace(namespace, input)?,
            background,
        })
    }

    pub fn new_with_identity_options<T: AsRef<[u8]>>(
        options: AvatarIdentityOptions<'_>,
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_options(options, input)?,
            background,
        })
    }
}

impl AvatarRenderer for HashedDogAvatar {
    fn render(&self, spec: AvatarSpec) -> Result<RgbaImage, AvatarSpecError> {
        render_dog_avatar_for_identity(spec, &self.identity, self.background)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HashedRobotAvatar {
    identity: AvatarIdentity,
    background: AvatarBackground,
}

impl HashedRobotAvatar {
    pub fn new<T: AsRef<[u8]>>(
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Self::new_with_namespace(AvatarNamespace::default(), input, background)
    }

    pub fn new_with_namespace<T: AsRef<[u8]>>(
        namespace: AvatarNamespace<'_>,
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_namespace(namespace, input)?,
            background,
        })
    }

    pub fn new_with_identity_options<T: AsRef<[u8]>>(
        options: AvatarIdentityOptions<'_>,
        input: T,
        background: AvatarBackground,
    ) -> Result<Self, AvatarIdentityError> {
        Ok(Self {
            identity: AvatarIdentity::new_with_options(options, input)?,
            background,
        })
    }
}

impl AvatarRenderer for HashedRobotAvatar {
    fn render(&self, spec: AvatarSpec) -> Result<RgbaImage, AvatarSpecError> {
        render_robot_avatar_for_identity(spec, &self.identity, self.background)
    }
}

