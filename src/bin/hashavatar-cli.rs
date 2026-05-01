use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use hashavatar::{
    AvatarBackground, AvatarKind, AvatarOptions, AvatarOutputFormat, AvatarSpec,
    MAX_AVATAR_DIMENSION, MIN_AVATAR_DIMENSION, encode_avatar_for_id, export_avatar_svg_for_id,
};

const MAX_IDENTITY_BYTES: usize = 1024;
const MAX_BATCH_INPUT_BYTES: u64 = 1_048_576;
const MAX_OUTPUT_BASENAME_CHARS: usize = 96;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse(std::env::args_os().skip(1).collect())?;
    let spec = AvatarSpec::new(args.size, args.size, 0);
    spec.validate()?;

    if let Some(input) = args.input.as_ref() {
        let out_dir = args
            .out_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("exports"));
        fs::create_dir_all(&out_dir)?;

        for identity in read_batch_identities(input)? {
            validate_identity(&identity)?;
            let output = out_dir.join(output_file_name(&identity, args.kind, args.format));
            export_one(
                &identity,
                spec,
                &output,
                args.kind,
                args.background,
                args.format,
            )?;
            println!("wrote {}", output.display());
        }
        return Ok(());
    }

    let identity = args
        .id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or("missing --id or --input")?;
    validate_identity(identity)?;
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from(output_file_name(identity, args.kind, args.format)));
    export_one(
        identity,
        spec,
        &output,
        args.kind,
        args.background,
        args.format,
    )?;
    println!("wrote {}", output.display());
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CliFormat {
    WebP,
    Png,
    Jpeg,
    Gif,
    Svg,
}

impl CliFormat {
    fn extension(self) -> &'static str {
        match self {
            Self::WebP => "webp",
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
            Self::Svg => "svg",
        }
    }
}

impl FromStr for CliFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "webp" => Ok(Self::WebP),
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            "gif" => Ok(Self::Gif),
            "svg" => Ok(Self::Svg),
            _ => Err("unsupported format"),
        }
    }
}

#[derive(Debug)]
struct CliArgs {
    id: Option<String>,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    out_dir: Option<PathBuf>,
    kind: AvatarKind,
    background: AvatarBackground,
    format: CliFormat,
    size: u32,
}

impl CliArgs {
    fn parse(args: Vec<OsString>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut parsed = Self {
            id: None,
            input: None,
            output: None,
            out_dir: None,
            kind: AvatarKind::Cat,
            background: AvatarBackground::Themed,
            format: CliFormat::WebP,
            size: 256,
        };

        let mut iter = args.into_iter();
        while let Some(flag) = iter.next() {
            let value = flag.to_string_lossy();
            match value.as_ref() {
                "--id" => parsed.id = Some(next_string(&mut iter, "--id")?),
                "--input" => parsed.input = Some(PathBuf::from(next_string(&mut iter, "--input")?)),
                "--output" => {
                    parsed.output = Some(PathBuf::from(next_string(&mut iter, "--output")?))
                }
                "--out-dir" => {
                    parsed.out_dir = Some(PathBuf::from(next_string(&mut iter, "--out-dir")?))
                }
                "--kind" => {
                    parsed.kind = AvatarKind::from_str(&next_string(&mut iter, "--kind")?)
                        .map_err(str::to_string)?
                }
                "--background" => {
                    parsed.background =
                        AvatarBackground::from_str(&next_string(&mut iter, "--background")?)
                            .map_err(str::to_string)?
                }
                "--format" => {
                    parsed.format = CliFormat::from_str(&next_string(&mut iter, "--format")?)
                        .map_err(str::to_string)?
                }
                "--size" => {
                    parsed.size = next_string(&mut iter, "--size")?.parse::<u32>()?;
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => return Err(format!("unknown argument: {other}").into()),
            }
        }

        Ok(parsed)
    }
}

fn next_string(
    iter: &mut impl Iterator<Item = OsString>,
    flag: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    iter.next()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| format!("missing value for {flag}").into())
}

fn read_batch_identities(path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if let Ok(metadata) = fs::metadata(path)
        && metadata.len() > MAX_BATCH_INPUT_BYTES
    {
        return Err(format!("input file is too large: max {MAX_BATCH_INPUT_BYTES} bytes").into());
    }

    let mut contents = String::new();
    let mut file = fs::File::open(path)?;
    let bytes_read = file
        .by_ref()
        .take(MAX_BATCH_INPUT_BYTES + 1)
        .read_to_string(&mut contents)?;
    if bytes_read as u64 > MAX_BATCH_INPUT_BYTES {
        return Err(format!("input file is too large: max {MAX_BATCH_INPUT_BYTES} bytes").into());
    }

    Ok(contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn validate_identity(identity: &str) -> Result<(), Box<dyn std::error::Error>> {
    if identity.len() > MAX_IDENTITY_BYTES {
        Err(format!("identity is too long: max {MAX_IDENTITY_BYTES} bytes").into())
    } else {
        Ok(())
    }
}

fn export_one(
    identity: &str,
    spec: AvatarSpec,
    output: &Path,
    kind: AvatarKind,
    background: AvatarBackground,
    format: CliFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let options = AvatarOptions::new(kind, background);
    match format {
        CliFormat::Svg => export_avatar_svg_for_id(spec, identity, options, output)?,
        CliFormat::WebP | CliFormat::Png | CliFormat::Jpeg | CliFormat::Gif => {
            let bytes = encode_avatar_for_id(spec, identity, format.into(), options)?;
            fs::write(output, bytes)?;
        }
    }
    Ok(())
}

impl From<CliFormat> for AvatarOutputFormat {
    fn from(value: CliFormat) -> Self {
        match value {
            CliFormat::WebP => Self::WebP,
            CliFormat::Png => Self::Png,
            CliFormat::Jpeg => Self::Jpeg,
            CliFormat::Gif => Self::Gif,
            CliFormat::Svg => unreachable!("SVG is handled outside AvatarOutputFormat"),
        }
    }
}

fn output_file_name(identity: &str, kind: AvatarKind, format: CliFormat) -> String {
    let safe_identity = identity
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => ch,
            _ => '-',
        })
        .take(MAX_OUTPUT_BASENAME_CHARS)
        .collect::<String>()
        .trim_matches('-')
        .to_ascii_lowercase();
    format!(
        "{}-{}.{}",
        if safe_identity.is_empty() {
            "avatar".to_string()
        } else {
            safe_identity
        },
        kind.as_str(),
        format.extension()
    )
}

fn print_help() {
    println!(
        "hashavatar-cli\n\
         \n\
         Single export:\n\
           cargo run --bin hashavatar-cli -- --id robot@hashavatar.app --kind robot --background transparent --format svg --output robot.svg\n\
         \n\
         Batch export:\n\
           cargo run --bin hashavatar-cli -- --input ids.txt --out-dir exports --kind fox --format webp\n\
         \n\
         Flags:\n\
           --id <value>\n\
           --input <path>\n\
           --output <path>\n\
           --out-dir <path>\n\
           --kind <cat|dog|robot|fox|alien|monster|ghost|slime|bird|wizard|skull|paws|planet|rocket|mushroom|cactus|frog|panda|cupcake|pizza|icecream|octopus|knight>\n\
           --background <themed|white|black|dark|light|transparent>\n\
           --format <webp|png|jpg|gif|svg>\n\
           --size <pixels> ({MIN_AVATAR_DIMENSION}..={MAX_AVATAR_DIMENSION})\n"
    );
}
