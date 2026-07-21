use sanitization::{Secret, SecretVec};
use sanitization_crypto_interop::sha2::sha512_digest;

use crate::{CatError, IdentityComponent, MAX_IDENTITY_BYTES, MAX_NAMESPACE_COMPONENT_BYTES};

const IDENTITY_DOMAIN: &[u8] = b"hashavatar/identity/v2/sha512/v1";
const TRAIT_DOMAIN: &[u8] = b"hashavatar/trait/v2/sha512/v1";
pub(crate) struct TraitDeriver {
    digest: Secret<[u8; 64]>,
    style_seed: u64,
}

impl TraitDeriver {
    pub(crate) fn with_namespace(
        tenant: &[u8],
        style_version: &[u8],
        input: &[u8],
        style_seed: u64,
    ) -> Result<Self, CatError> {
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
            .ok_or(CatError::NumericRange)?;
        let mut preimage =
            SecretVec::try_with_capacity(capacity).map_err(|_| CatError::Allocation)?;
        append_component(&mut preimage, IDENTITY_DOMAIN)?;
        append_component(&mut preimage, tenant)?;
        append_component(&mut preimage, style_version)?;
        append_component(&mut preimage, input)?;
        let digest = Secret::new(preimage.with_secret(sha512_digest));
        Ok(Self { digest, style_seed })
    }

    pub(crate) fn sample(&self, label: &[u8]) -> Result<u16, CatError> {
        self.sample_components(None, label)
    }

    pub(crate) fn sample_scoped(&self, scope: &[u8], label: &[u8]) -> Result<u16, CatError> {
        self.sample_components(Some(scope), label)
    }

    fn sample_components(&self, scope: Option<&[u8]>, label: &[u8]) -> Result<u16, CatError> {
        let seed = self.style_seed.to_le_bytes();
        let counter = 0_u32.to_le_bytes();
        let mut capacity = component_size(TRAIT_DOMAIN)?
            .checked_add(component_size(&[0_u8; 64])?)
            .and_then(|value| value.checked_add(component_size(&seed).ok()?))
            .and_then(|value| value.checked_add(component_size(label).ok()?))
            .and_then(|value| value.checked_add(component_size(&counter).ok()?))
            .ok_or(CatError::NumericRange)?;
        if let Some(scope) = scope {
            capacity = capacity
                .checked_add(component_size(scope)?)
                .ok_or(CatError::NumericRange)?;
        }
        let mut preimage =
            SecretVec::try_with_capacity(capacity).map_err(|_| CatError::Allocation)?;
        append_component(&mut preimage, TRAIT_DOMAIN)?;
        self.digest
            .with_secret(|digest| append_component(&mut preimage, digest))?;
        append_component(&mut preimage, &seed)?;
        if let Some(scope) = scope {
            append_component(&mut preimage, scope)?;
        }
        append_component(&mut preimage, label)?;
        append_component(&mut preimage, &counter)?;
        let digest = Secret::new(preimage.with_secret(sha512_digest));
        digest.with_secret(|digest| {
            let first = digest.first().copied().ok_or(CatError::NumericRange)?;
            let second = digest.get(1).copied().ok_or(CatError::NumericRange)?;
            Ok(u16::from_le_bytes([first, second]))
        })
    }
}

fn validate_component(
    component: IdentityComponent,
    bytes: &[u8],
    maximum: usize,
) -> Result<(), CatError> {
    if bytes.len() <= maximum {
        Ok(())
    } else {
        Err(CatError::IdentityComponentTooLong { component, maximum })
    }
}

fn component_size(bytes: &[u8]) -> Result<usize, CatError> {
    core::mem::size_of::<u64>()
        .checked_add(bytes.len())
        .ok_or(CatError::NumericRange)
}

fn append_component(preimage: &mut SecretVec, bytes: &[u8]) -> Result<(), CatError> {
    let length = u64::try_from(bytes.len()).map_err(|_| CatError::NumericRange)?;
    preimage.extend_from_slice(&length.to_le_bytes());
    preimage.extend_from_slice(bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_independent_and_stable() -> Result<(), CatError> {
        let deriver = TraitDeriver::with_namespace(b"public", b"v2-alpha2", b"alpha-cat", 7)?;
        assert_eq!(deriver.sample(b"head-width"), deriver.sample(b"head-width"));
        assert_ne!(
            deriver.sample(b"head-width"),
            deriver.sample(b"head-height")
        );
        Ok(())
    }

    #[test]
    fn namespace_components_are_unambiguous() -> Result<(), CatError> {
        let a = TraitDeriver::with_namespace(b"a\0b", b"c", b"id", 0)?;
        let b = TraitDeriver::with_namespace(b"a", b"b\0c", b"id", 0)?;
        assert_ne!(a.sample(b"head-width"), b.sample(b"head-width"));
        Ok(())
    }
}
