#![no_main]

use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_for_id, render_avatar_svg_for_id,
};
use hashavatar_core::{
    AvatarHashAlgorithm as CoreHashAlgorithm, AvatarIdentityOptions as CoreIdentityOptions,
    AvatarNamespace as CoreNamespace, AvatarOptions as CoreOptions,
    AvatarRenderPlan as CoreRenderPlan, AvatarSpec as CoreSpec,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let kind = AvatarKind::from_byte(data[0]);
    let background = AvatarBackground::from_byte(*data.get(1).unwrap_or(&0));
    let size = 64 + u32::from(*data.get(2).unwrap_or(&0) % 8) * 64;
    let identity_len = data.len().min(128);
    let identity = &data[..identity_len];
    let Ok(spec) = AvatarSpec::new(size, size, 0) else {
        return;
    };
    let options = AvatarOptions::new(kind, background);

    let _ = render_avatar_svg_for_id(spec, identity, options);
    let _ = encode_avatar_for_id(spec, identity, AvatarOutputFormat::Png, options);

    if let Ok(core_spec) = CoreSpec::new(size, size, 0) {
        let core_namespace = CoreNamespace::new("fuzz", "v2").expect("static namespace is valid");
        let core_identity_options =
            CoreIdentityOptions::new(core_namespace, CoreHashAlgorithm::Sha512);
        let core_options = CoreOptions::new(
            hashavatar_core::AvatarKind::from_byte(data[0]),
            hashavatar_core::AvatarBackground::from_byte(*data.get(1).unwrap_or(&0)),
        );
        let _ = CoreRenderPlan::new(core_spec, core_identity_options, identity, core_options);
    }
});
