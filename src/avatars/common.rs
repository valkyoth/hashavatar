fn seeded_renderer_rng(spec: AvatarSpec, identity: &AvatarIdentity) -> StdRng {
    let mut rng_seed = identity.rng_seed();
    rng_seed.with_secret_mut(|rng_seed| {
        for (index, byte) in spec.seed.to_le_bytes().iter().enumerate() {
            rng_seed[index] ^= *byte;
        }
    });
    let rng_seed_value = Secret::new(rng_seed.with_secret(|rng_seed| *rng_seed));
    drop(rng_seed);
    rng_seed_value.with_secret(|rng_seed_value| StdRng::from_seed(*rng_seed_value))
}
