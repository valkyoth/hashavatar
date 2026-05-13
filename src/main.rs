use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;

use axum::Router;
use axum::extract::Query;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    encode_avatar_for_id,
};

const DEFAULT_IDENTITY: &str = "cat@hashavatar.app";
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";
const MAX_QUERY_IDENTITY_BYTES: usize = 512;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(index))
        .route("/avatar.webp", get(avatar_webp));

    let address = std::env::var("HASHAVATAR_BIND_ADDR")
        .unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_owned())
        .parse::<SocketAddr>()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    let local_address = listener.local_addr()?;

    println!(
        "hashavatar demo listening on {}",
        log_safe(format!("http://{local_address}"))
    );

    axum::serve(listener, app).await?;
    Ok(())
}

fn log_safe(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .chars()
        .flat_map(char::escape_default)
        .collect()
}

async fn index() -> Html<String> {
    Html(render_index_html(DEFAULT_IDENTITY))
}

async fn avatar_webp(Query(params): Query<HashMap<String, String>>) -> Response {
    let identity = params
        .get("id")
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(DEFAULT_IDENTITY);
    if identity.len() > MAX_QUERY_IDENTITY_BYTES {
        return (
            StatusCode::BAD_REQUEST,
            format!("identity is too long: max {MAX_QUERY_IDENTITY_BYTES} bytes"),
        )
            .into_response();
    }

    let kind = parse_kind(params.get("kind").map(String::as_str));
    let background = parse_background(params.get("background").map(String::as_str));

    match encode_avatar_for_id(
        AvatarSpec::new(256, 256, 0),
        identity,
        AvatarOutputFormat::WebP,
        AvatarOptions { kind, background },
    ) {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/webp"));
            headers.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("no-store, max-age=0"),
            );
            (StatusCode::OK, headers, bytes).into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("avatar generation failed: {error}"),
        )
            .into_response(),
    }
}

fn parse_kind(value: Option<&str>) -> AvatarKind {
    value
        .and_then(|raw| AvatarKind::from_str(raw).ok())
        .unwrap_or(AvatarKind::Cat)
}

fn parse_background(value: Option<&str>) -> AvatarBackground {
    value
        .and_then(|raw| AvatarBackground::from_str(raw).ok())
        .unwrap_or(AvatarBackground::Themed)
}

fn render_index_html(default_identity: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Avatar Generator Demo</title>
  <style>
    :root {{
      --bg: #f4efe8;
      --panel: rgba(255, 252, 247, 0.92);
      --ink: #1f2933;
      --muted: #52606d;
      --line: rgba(31, 41, 51, 0.1);
      --accent: #d97a42;
      --accent-strong: #b65a28;
      --shadow: 0 24px 80px rgba(75, 48, 25, 0.16);
      --radius: 28px;
      font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
    }}

    * {{
      box-sizing: border-box;
    }}

    body {{
      margin: 0;
      min-height: 100vh;
      color: var(--ink);
      background:
        radial-gradient(circle at top left, rgba(255, 214, 170, 0.9), transparent 26%),
        radial-gradient(circle at bottom right, rgba(217, 122, 66, 0.18), transparent 30%),
        linear-gradient(135deg, #fbf6ee, var(--bg));
      display: grid;
      place-items: center;
      padding: 24px;
    }}

    .shell {{
      width: min(980px, 100%);
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: calc(var(--radius) + 8px);
      box-shadow: var(--shadow);
      overflow: hidden;
      backdrop-filter: blur(16px);
    }}

    .grid {{
      display: grid;
      grid-template-columns: 1.1fr 0.9fr;
    }}

    .copy, .preview {{
      padding: 36px;
    }}

    .copy {{
      border-right: 1px solid var(--line);
    }}

    .eyebrow {{
      display: inline-flex;
      gap: 10px;
      align-items: center;
      font-size: 0.78rem;
      font-weight: 700;
      letter-spacing: 0.12em;
      text-transform: uppercase;
      color: var(--accent-strong);
    }}

    .eyebrow::before {{
      content: "";
      width: 28px;
      height: 1px;
      background: currentColor;
    }}

    h1 {{
      margin: 18px 0 14px;
      font-size: clamp(2.2rem, 6vw, 4.3rem);
      line-height: 0.94;
      letter-spacing: -0.05em;
      max-width: 11ch;
    }}

    p {{
      margin: 0;
      color: var(--muted);
      line-height: 1.6;
      max-width: 58ch;
    }}

    form {{
      margin-top: 28px;
      display: grid;
      gap: 16px;
    }}

    label {{
      display: block;
      margin-bottom: 8px;
      font-size: 0.92rem;
      font-weight: 700;
      color: var(--ink);
    }}

    .controls {{
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 12px;
    }}

    .picker-grid {{
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 14px;
    }}

    input, select {{
      width: 100%;
      border: 1px solid rgba(82, 96, 109, 0.18);
      background: rgba(255, 255, 255, 0.9);
      color: var(--ink);
      border-radius: 16px;
      padding: 15px 18px;
      font: inherit;
      outline: none;
      transition: border-color 160ms ease, box-shadow 160ms ease, transform 160ms ease;
    }}

    input:focus, select:focus {{
      border-color: rgba(217, 122, 66, 0.65);
      box-shadow: 0 0 0 5px rgba(217, 122, 66, 0.12);
      transform: translateY(-1px);
    }}

    button {{
      border: 0;
      border-radius: 16px;
      padding: 0 20px;
      min-height: 52px;
      background: linear-gradient(180deg, #dd8750, #c96831);
      color: white;
      font: inherit;
      font-weight: 700;
      cursor: pointer;
      box-shadow: 0 16px 32px rgba(201, 104, 49, 0.24);
    }}

    .hint {{
      font-size: 0.88rem;
      color: var(--muted);
    }}

    .preview {{
      display: grid;
      place-items: center;
      gap: 18px;
      background:
        radial-gradient(circle at center, rgba(255,255,255,0.74), rgba(255,255,255,0) 62%),
        linear-gradient(180deg, rgba(255,255,255,0.5), rgba(255,255,255,0.15));
    }}

    .avatar-frame {{
      width: min(320px, 72vw);
      aspect-ratio: 1;
      display: grid;
      place-items: center;
      border-radius: 32px;
      background:
        linear-gradient(180deg, rgba(255,255,255,0.95), rgba(255,255,255,0.72)),
        linear-gradient(135deg, rgba(217, 122, 66, 0.16), rgba(255, 214, 170, 0.12));
      border: 1px solid rgba(255,255,255,0.8);
      box-shadow: inset 0 1px 0 rgba(255,255,255,0.8), 0 18px 40px rgba(82, 96, 109, 0.12);
    }}

    img {{
      width: min(256px, 100%);
      height: auto;
      display: block;
    }}

    .caption {{
      width: 100%;
      padding: 16px 18px;
      border-radius: 18px;
      background: rgba(255,255,255,0.74);
      border: 1px solid rgba(31, 41, 51, 0.08);
      font-size: 0.92rem;
      color: var(--muted);
      display: grid;
      gap: 6px;
    }}

    .caption strong {{
      color: var(--ink);
    }}

    @media (max-width: 820px) {{
      .grid {{
        grid-template-columns: 1fr;
      }}

      .copy {{
        border-right: 0;
        border-bottom: 1px solid var(--line);
      }}

      .copy, .preview {{
        padding: 24px;
      }}

      .controls, .picker-grid {{
        grid-template-columns: 1fr;
      }}
    }}
  </style>
</head>
<body>
  <main class="shell">
    <section class="grid">
      <div class="copy">
        <div class="eyebrow">Rust Demo</div>
        <h1>Choose The Avatar Personality</h1>
        <p>
          Type an email, username, or stable identifier, then choose an avatar family.
          Backgrounds can be themed, white, black, dark, light, or transparent for compositing.
        </p>

        <form id="avatar-form">
          <div>
            <label for="identity">Identity</label>
            <div class="controls">
              <input
                id="identity"
                name="identity"
                type="text"
                value="{default_identity}"
                placeholder="cat@hashavatar.app"
                autocomplete="off"
                spellcheck="false"
              />
              <button type="submit">Generate</button>
            </div>
          </div>

          <div class="picker-grid">
            <div>
              <label for="kind">Avatar Type</label>
              <select id="kind" name="kind">
                <option value="cat" data-identity="cat@hashavatar.app" selected>Cat</option>
                <option value="dog" data-identity="dog@hashavatar.app">Dog</option>
                <option value="robot" data-identity="robot@hashavatar.app">Robot</option>
                <option value="fox" data-identity="fox@hashavatar.app">Fox</option>
                <option value="alien" data-identity="alien@hashavatar.app">Alien</option>
                <option value="monster" data-identity="monster@hashavatar.app">Monster</option>
                <option value="ghost" data-identity="ghost@hashavatar.app">Ghost</option>
                <option value="slime" data-identity="slime@hashavatar.app">Slime</option>
                <option value="bird" data-identity="bird@hashavatar.app">Bird</option>
                <option value="wizard" data-identity="wizard@hashavatar.app">Wizard</option>
                <option value="skull" data-identity="skull@hashavatar.app">Skull</option>
                <option value="paws" data-identity="paws@hashavatar.app">Paws</option>
                <option value="planet" data-identity="planet@hashavatar.app">Planet</option>
                <option value="rocket" data-identity="rocket@hashavatar.app">Rocket</option>
                <option value="mushroom" data-identity="mushroom@hashavatar.app">Mushroom</option>
                <option value="cactus" data-identity="cactus@hashavatar.app">Cactus</option>
                <option value="frog" data-identity="frog@hashavatar.app">Frog</option>
                <option value="panda" data-identity="panda@hashavatar.app">Panda</option>
                <option value="cupcake" data-identity="cupcake@hashavatar.app">Cupcake</option>
                <option value="pizza" data-identity="pizza@hashavatar.app">Pizza</option>
                <option value="icecream" data-identity="icecream@hashavatar.app">Ice Cream</option>
                <option value="octopus" data-identity="octopus@hashavatar.app">Octopus</option>
                <option value="knight" data-identity="knight@hashavatar.app">Knight</option>
              </select>
            </div>
            <div>
              <label for="background">Background</label>
              <select id="background" name="background">
                <option value="themed" selected>Themed</option>
                <option value="white">White</option>
                <option value="black">Black</option>
                <option value="dark">Dark</option>
                <option value="light">Light</option>
                <option value="transparent">Transparent</option>
              </select>
            </div>
          </div>

          <div class="hint">WebP is still used by default for smaller avatar payloads.</div>
        </form>
      </div>

      <div class="preview">
        <div class="avatar-frame">
          <img
            id="avatar-image"
            src="/avatar.webp?id={default_identity}&kind=cat&background=themed"
            alt="Generated avatar preview"
            width="256"
            height="256"
          />
        </div>
        <div class="caption">
          <div><strong>Current identity:</strong> <span id="identity-readout">{default_identity}</span></div>
          <div><strong>Avatar type:</strong> <span id="kind-readout">cat</span></div>
          <div><strong>Background:</strong> <span id="background-readout">themed</span></div>
        </div>
      </div>
    </section>
  </main>

  <script>
    const form = document.getElementById("avatar-form");
    const input = document.getElementById("identity");
    const kind = document.getElementById("kind");
    const background = document.getElementById("background");
    const image = document.getElementById("avatar-image");
    const identityReadout = document.getElementById("identity-readout");
    const kindReadout = document.getElementById("kind-readout");
    const backgroundReadout = document.getElementById("background-readout");
    const presetIdentities = new Map(
      Array.from(kind.options).map((option) => [option.value, option.dataset.identity])
    );

    function selectedPresetIdentity() {{
      return presetIdentities.get(kind.value) || "{default_identity}";
    }}

    function isPresetIdentity(value) {{
      for (const identity of presetIdentities.values()) {{
        if (value === identity) {{
          return true;
        }}
      }}
      return false;
    }}

    function refreshAvatar() {{
      const identity = input.value.trim() || selectedPresetIdentity();
      const avatarKind = kind.value;
      const bg = background.value;
      const query = new URLSearchParams({{
        id: identity,
        kind: avatarKind,
        background: bg,
        ts: String(Date.now()),
      }});

      image.src = `/avatar.webp?${{query.toString()}}`;
      identityReadout.textContent = identity;
      kindReadout.textContent = avatarKind;
      backgroundReadout.textContent = bg;
    }}

    form.addEventListener("submit", (event) => {{
      event.preventDefault();
      refreshAvatar();
    }});

    kind.addEventListener("change", () => {{
      const currentIdentity = input.value.trim();
      if (currentIdentity === "" || isPresetIdentity(currentIdentity)) {{
        input.value = selectedPresetIdentity();
      }}
      refreshAvatar();
    }});

    background.addEventListener("change", refreshAvatar);

    input.addEventListener("keydown", (event) => {{
      if (event.key === "Enter") {{
        event.preventDefault();
        refreshAvatar();
      }}
    }});
  </script>
</body>
</html>
"#
    )
}
