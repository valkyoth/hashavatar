# Crate Version Matrix

The machine-readable release decision is [`release-crates.toml`](../release-crates.toml).

| Package | Current source version | crates.io state | Role |
| --- | --- | --- | --- |
| `hashavatar-core` | `0.1.0-alpha.1` | Source-only; never published for this alpha | Portable canonical Cat core |
| `hashavatar` | `2.0.0-alpha.1` | Source-only; crates.io remains `1.3.0` | Recommended facade |

Dependencies precede dependants in `publish_order`. Every workspace package
must have an explicit version, change classification, publication decision,
and reason. The release tool rejects `publish = true` for alpha, beta, and RC
versions. Those versions identify implementation-stop commits, not Git tags.

At stable 2.0, changed publishable packages are published in dependency order
only after a signed tag and permanent pentest report exist. Unchanged packages
remain explicit with `change = "unchanged"` and `publish = false`.
