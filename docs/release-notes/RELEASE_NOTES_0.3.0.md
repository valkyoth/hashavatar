# Release Notes 0.3.0

`0.3.0` turns `hashavatar` into a fuller public-avatar platform with stronger renderer coverage, clearer stability rules, and a much more complete public API surface.

## Highlights

- New families: `ghost`, `slime`, `bird`, `wizard`, `skull`, `paws`
- Namespace-aware identities through `AvatarNamespace`
- Declared style contract through `AVATAR_STYLE_VERSION`
- Golden visual regression fixtures
- Hardened SVG regression coverage
- Public API docs and OpenAPI description
- Rate limiting, metrics, and timeout handling in the API layer
- S3/object-storage deduplication before upload
- SEO improvements, JSON-LD, sitemap, robots, favicon, manifest, and OG preview support

## Upgrade Guidance

- If you want stable output separation between apps or rollout phases, start passing `tenant` and `style_version`
- If you operate the API publicly, deploy behind Cloudflare or another CDN and keep aggressive caching limited to avatar asset routes
- Treat `0.3.0` as the point where rendering stability becomes an explicit documented contract and visual regression testing is part of the normal workflow
