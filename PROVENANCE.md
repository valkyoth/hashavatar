# Provenance

This repository is intended to be a code-generated avatar system.

## Asset statement

- No raster sprite sheets are bundled with the crate.
- No third-party character packs are bundled with the crate.
- No licensed avatar art is embedded in the crate as source assets.
- Avatar output is drawn procedurally from Rust code using geometric primitives.

## Generation model

- Identity input is hashed with `SHA-512`.
- Digest bytes are mapped into visual parameters such as proportions, colors, spacing, and facial details.
- Final images are rendered using first-party geometric primitives and the
  `image` crate's RGBA buffers and encoders.

## Current avatar families

- Cat, dog, robot, fox, alien, monster, ghost, slime, bird, wizard, and skull
- Paws, planet, rocket, mushroom, cactus, frog, panda, cupcake, pizza, ice
  cream, octopus, and knight
- Bear, penguin, dragon, ninja, astronaut, diamond, coffee cup, and shield

## Background modes

- Themed, white, black, dark, light, and transparent
- Polka dot, striped, checkerboard, grid, sunrise, ocean, and starry

## Practical implication

The repository is materially different from avatar systems that depend on pre-made asset packs. The visuals are generated from code at runtime, which avoids the usual licensing concerns around bundled image libraries. This file is not legal advice; it is a technical provenance statement about how the repository is constructed.

## Output formats

- Lossless WebP is available in default builds.
- PNG, JPEG, and GIF are explicit opt-in Cargo features.
- SVG string rendering is available without an encoder feature.
