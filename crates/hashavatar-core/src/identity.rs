use sanitization::{Secret, SecretVec};
use sanitization_crypto_interop::sha2::sha512_digest;

use crate::{
    AvatarError, DEFAULT_STYLE_VERSION, DEFAULT_TENANT, IdentityCacheKey, IdentityComponent,
    MAX_IDENTITY_BYTES, MAX_NAMESPACE_COMPONENT_BYTES,
};

const IDENTITY_DOMAIN: &[u8] = b"hashavatar/identity/v2/sha512/v1";
const TRAIT_DOMAIN: &[u8] = b"hashavatar/trait/v2/sha512/v1";
/// Validated, domain-separated avatar identity.
///
/// The raw identifier and namespace are hashed during construction and are not
/// retained. Debug output is redacted and the digest is sanitized on drop.
#[must_use = "pass the identity to AvatarRequest::builder or AvatarRequest::from_identity"]
pub struct AvatarIdentity {
    digest: Secret<[u8; 64]>,
}

impl AvatarIdentity {
    /// Derives an identity in the default public namespace.
    pub fn new(input: impl AsRef<[u8]>) -> Result<Self, AvatarError> {
        Self::with_namespace(DEFAULT_TENANT, DEFAULT_STYLE_VERSION, input)
    }

    /// Derives an identity from bounded, length-prefixed namespace components.
    pub fn with_namespace(
        tenant: impl AsRef<[u8]>,
        style_version: impl AsRef<[u8]>,
        input: impl AsRef<[u8]>,
    ) -> Result<Self, AvatarError> {
        let tenant = tenant.as_ref();
        let style_version = style_version.as_ref();
        let input = input.as_ref();
        validate_component(IdentityComponent::Input, input, MAX_IDENTITY_BYTES)?;
        validate_component(
            IdentityComponent::Tenant,
            tenant,
            MAX_NAMESPACE_COMPONENT_BYTES,
        )?;
        validate_component(
            IdentityComponent::StyleVersion,
            style_version,
            MAX_NAMESPACE_COMPONENT_BYTES,
        )?;

        let capacity = component_size(IDENTITY_DOMAIN)?
            .checked_add(component_size(tenant)?)
            .and_then(|value| value.checked_add(component_size(style_version).ok()?))
            .and_then(|value| value.checked_add(component_size(input).ok()?))
            .ok_or(AvatarError::NumericRange)?;
        let mut preimage =
            SecretVec::try_with_capacity(capacity).map_err(|_| AvatarError::Allocation)?;
        append_component(&mut preimage, IDENTITY_DOMAIN)?;
        append_component(&mut preimage, tenant)?;
        append_component(&mut preimage, style_version)?;
        append_component(&mut preimage, input)?;
        Ok(Self {
            digest: Secret::new(preimage.with_secret(sha512_digest)),
        })
    }

    /// Returns a public, correlatable, domain-separated identity cache key.
    pub fn cache_key(&self) -> Result<IdentityCacheKey, AvatarError> {
        crate::keys::identity_cache_key(self)
    }

    pub(crate) fn with_digest<R>(&self, inspect: impl FnOnce(&[u8; 64]) -> R) -> R {
        self.digest.with_secret(inspect)
    }
}

impl core::fmt::Debug for AvatarIdentity {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("AvatarIdentity([REDACTED])")
    }
}

pub(crate) struct TraitDeriver {
    identity: AvatarIdentity,
    style_seed: u64,
}

impl TraitDeriver {
    pub(crate) const fn new(identity: AvatarIdentity, style_seed: u64) -> Self {
        Self {
            identity,
            style_seed,
        }
    }

    pub(crate) fn sample(&self, label: &[u8]) -> Result<u16, AvatarError> {
        self.sample_components(None, label)
    }

    pub(crate) fn sample_scoped(&self, scope: &[u8], label: &[u8]) -> Result<u16, AvatarError> {
        self.sample_components(Some(scope), label)
    }

    fn sample_components(&self, scope: Option<&[u8]>, label: &[u8]) -> Result<u16, AvatarError> {
        let seed = self.style_seed.to_le_bytes();
        let counter = 0_u32.to_le_bytes();
        let mut capacity = component_size(TRAIT_DOMAIN)?
            .checked_add(component_size(&[0_u8; 64])?)
            .and_then(|value| value.checked_add(component_size(&seed).ok()?))
            .and_then(|value| value.checked_add(component_size(label).ok()?))
            .and_then(|value| value.checked_add(component_size(&counter).ok()?))
            .ok_or(AvatarError::NumericRange)?;
        if let Some(scope) = scope {
            capacity = capacity
                .checked_add(component_size(scope)?)
                .ok_or(AvatarError::NumericRange)?;
        }
        let mut preimage =
            SecretVec::try_with_capacity(capacity).map_err(|_| AvatarError::Allocation)?;
        append_component(&mut preimage, TRAIT_DOMAIN)?;
        self.identity
            .with_digest(|digest| append_component(&mut preimage, digest))?;
        append_component(&mut preimage, &seed)?;
        if let Some(scope) = scope {
            append_component(&mut preimage, scope)?;
        }
        append_component(&mut preimage, label)?;
        append_component(&mut preimage, &counter)?;
        let digest = Secret::new(preimage.with_secret(sha512_digest));
        digest.with_secret(|digest| {
            let first = digest.first().copied().ok_or(AvatarError::NumericRange)?;
            let second = digest.get(1).copied().ok_or(AvatarError::NumericRange)?;
            Ok(u16::from_le_bytes([first, second]))
        })
    }

    pub(crate) const fn identity(&self) -> &AvatarIdentity {
        &self.identity
    }
}

fn validate_component(
    component: IdentityComponent,
    bytes: &[u8],
    maximum: usize,
) -> Result<(), AvatarError> {
    if bytes.len() <= maximum {
        Ok(())
    } else {
        Err(AvatarError::IdentityComponentTooLong { component, maximum })
    }
}

fn component_size(bytes: &[u8]) -> Result<usize, AvatarError> {
    core::mem::size_of::<u64>()
        .checked_add(bytes.len())
        .ok_or(AvatarError::NumericRange)
}

fn append_component(preimage: &mut SecretVec, bytes: &[u8]) -> Result<(), AvatarError> {
    let length = u64::try_from(bytes.len()).map_err(|_| AvatarError::NumericRange)?;
    preimage.extend_from_slice(&length.to_le_bytes());
    preimage.extend_from_slice(bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_independent_and_stable() -> Result<(), AvatarError> {
        let deriver = TraitDeriver::new(
            AvatarIdentity::with_namespace(b"public", b"v2-alpha2", b"alpha-cat")?,
            7,
        );
        assert_eq!(deriver.sample(b"head-width"), deriver.sample(b"head-width"));
        assert_ne!(
            deriver.sample(b"head-width"),
            deriver.sample(b"head-height")
        );
        Ok(())
    }

    #[test]
    fn namespace_components_are_unambiguous() -> Result<(), AvatarError> {
        let a = TraitDeriver::new(AvatarIdentity::with_namespace(b"a\0b", b"c", b"id")?, 0);
        let b = TraitDeriver::new(AvatarIdentity::with_namespace(b"a", b"b\0c", b"id")?, 0);
        assert_ne!(a.sample(b"head-width"), b.sample(b"head-width"));
        Ok(())
    }
}
