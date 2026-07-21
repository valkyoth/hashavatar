use crate::{MAX_SVG_OUTPUT_BYTES, SceneReport};

/// Conservative resource information bound to one prepared avatar.
///
/// This describes per-request core work and storage. Process-wide concurrency,
/// codec-owned memory, network buffers, and application caches remain caller
/// policy.
#[must_use = "use the budget for service admission and storage planning"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceBudget {
    scene: SceneReport,
}

impl ResourceBudget {
    pub(crate) const fn new(scene: SceneReport) -> Self {
        Self { scene }
    }

    /// Returns the underlying validated canonical scene report.
    pub const fn scene_report(self) -> SceneReport {
        self.scene
    }

    /// Returns exact tightly packed RGBA8 output and reusable-buffer bytes.
    pub const fn canonical_rgba_bytes(self) -> usize {
        self.scene.rgba_bytes()
    }

    /// Returns the conservative CPU candidate-pixel and active-clip tests.
    pub const fn estimated_pixel_tests(self) -> u64 {
        self.scene.estimated_pixel_tests()
    }

    /// Returns the maximum owned SVG output bytes admitted by core.
    pub const fn maximum_owned_svg_bytes(self) -> usize {
        MAX_SVG_OUTPUT_BYTES
    }
}
