# Layered Style Contract

Alpha.4 adds bounded, deterministic visual composition to the canonical scene.
No layered operation bypasses scene validation or has a separate raster/SVG
implementation.

## Requested Style

`AvatarStyle::new` creates a strict explicit style with the default palette,
default expression, and an empty accessory stack. This preserves alpha.3
layer-free output for the same complete request tuple.

`AccessoryStack` stores at most four accessories without allocation. Exceeding
`MAX_ACCESSORY_LAYERS` returns `AvatarError::AccessoryCapacity` without partial
growth. Accessories have frozen identifiers and typed slots:

| Slot | Admitted built-ins |
| --- | --- |
| Aura | halo |
| Headwear | hat, crown, horns |
| Earwear | headphones |
| Facewear | eyepatch |
| Eyewear | glasses |
| Neckwear | bowtie, scarf |

Back, handheld, and foreground slots are reserved semantic identifiers. They
do not claim admitted built-in accessories in alpha.4.

## Resolution

Explicit styles use `StyleResolutionPolicy::Strict` by default. A family
without face anchors rejects non-default expressions and accessories. Two
accessories in one slot are rejected, as are simultaneous facewear and
eyewear. `Cool` is an eyewear-like expression and conflicts with either of
those slots. All failures are typed and occur before scene execution.

`AutomaticFallback` first canonicalizes requests by slot and accessory ID.
It then uses the frozen `AvatarAccessory::ALL` order to replace a conflicting
layer with the first supported unoccupied non-colliding slot. If no candidate
exists, the request is explicitly reported as rejected. Unsupported
expressions become `Default`; a conflicting `Cool` expression becomes `Happy`.

Halo plus headwear and headwear plus earwear are both admitted with fixed
vertical adjustments. Family-specific corrections are integer basis-point
transforms. Resolution never depends on caller insertion order, hash-map order,
floating point, platform state, or mutable RNG state.

`AvatarStyle::automatic` derives two accessory requests, one expression, and
one palette from independent labeled trait samples, then applies the same
fallback policy.

## Reports And Ordering

`PreparedAvatar::resolved_style()` returns the exact effective palette,
integer color roles, expression, and accessories consumed by compilation.
`layout_report()` returns family anchors plus every accessory and expression
decision as accepted, adjusted, substituted, or rejected.

Accepted layers are sorted by `(z-band, slot ID, accessory ID)`. Halo and horns
use the behind-subject band, expressions use the expression band, and other
accessories use the foreground band. Reordering equivalent input stacks cannot
change the resolved style, report, scene, raster pixels, or SVG.

## Capabilities

`AvatarKind::capabilities()` is authoritative. Face-capable families admit all
currently populated accessory slots and expressions. Object/symbol families
without face anchors admit palettes, backgrounds, and frames but reject manual
face layers. Automatic attempts are reported rather than silently skipped.

## Assurance

Integration tests cover every family against every palette, expression, and
accessory; strict invalid cases; automatic fallback; permutation invariance;
maximum-capacity stacks; minimum/default/maximum dimensions; pixel-distinct
catalog choices; parsed SVG; and the complete alpha.3 baseline KAT. Fuzzing
compares forward and reverse stack resolution, and Kani proves fail-closed
fixed-capacity admission.
