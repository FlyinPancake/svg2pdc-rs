use anyhow::Result;
use clap::Parser;
use color::TruncateColor;
use std::path::{Path, PathBuf};
use svg_converter::SvgConverter;

mod color;
mod error;
mod image;
mod point;
mod svg_converter;

use error::{Svg2PdcError, Svg2PdcResult};
use point::{Conversion, Precision};

#[expect(clippy::too_many_arguments)]
fn create_pdc_from_path(
    input: &Path,
    output: &Path,
    precision: &Precision,
    truncate_color: &TruncateColor,
    conversion: &Conversion,
    verbose: bool,
    sequence: bool,
    #[expect(unused_variables)] duration: f32,
    #[expect(unused_variables)] play_count: u32,
) -> Svg2PdcResult<()> {
    if sequence {
        return Err(Svg2PdcError::UnsupportedOperation("sequence".to_string()));
    }

    let converter = SvgConverter::new(*precision);
    if input.exists() {
        if sequence {
            unreachable!();
        }

        if verbose {
            println!("Converting SVG file: {:?}", input);
        }

        // let dir_name = if input.is_dir() {
        //     input.to_path_buf()
        // } else {
        //     input.parent().unwrap().to_path_buf()
        // };

        // let frames = vec![];
        // let commands = vec![];

        if input.is_file() {
            let content = std::fs::read_to_string(input)?;

            let image = converter.parse_svg_image(&content, truncate_color, conversion)?;
            if verbose {
                image.inspect();
            }

            let output = if output.is_dir() {
                output
                    .join(input.file_stem().unwrap())
                    .with_extension("pdc")
            } else {
                output.to_path_buf()
            };

            let mut file = std::fs::File::create(output)?;
            image.serialize(&mut file)?;
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(version, about)]
struct Args {
    #[clap()]
    /// Input file
    input: PathBuf,

    #[clap(short, long)]
    /// Output file
    output: Option<PathBuf>,

    #[clap(short, long)]
    /// Use precise coordinates for path-like objects
    precise: bool,

    #[clap(short, long)]
    /// Create a sequence CURRENTLY UNSUPPORTED
    sequence: bool,

    #[clap(short, long)]
    truncate_color: bool,

    #[clap(short, long)]
    /// Duration of the animation in seconds CURRENTLY UNSUPPORTED
    duration: Option<f32>,

    #[clap(short, long)]
    /// Verbose output
    verbose: bool,

    #[clap(short, long)]
    /// Convert coordinates to Pebble's format
    convert: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let precision = if args.precise {
        Precision::Precise
    } else {
        Precision::Normal
    };

    let sequence = args.sequence;

    if sequence && args.duration.is_none() {
        return Err(Svg2PdcError::UnsupportedOperation("sequence".to_string()).into());
    }

    if !sequence && args.duration.is_some() {
        return Err(Svg2PdcError::UnsupportedOperation("duration".to_string()).into());
    }

    let truncate_color = if args.truncate_color {
        TruncateColor::Truncate
    } else {
        TruncateColor::Keep
    };

    let conversion = if args.convert {
        if args.verbose {
            Conversion::ConvertWarn
        } else {
            Conversion::ConvertNoWarn
        }
    } else {
        Conversion::RequireExact
    };

    let duration = args.duration.unwrap_or(0.0);

    let verbose = args.verbose;
    let input = args.input;
    let output = args.output.unwrap_or_else(|| input.with_extension("pdc"));
    let play_count = 1;

    create_pdc_from_path(
        &input,
        &output,
        &precision,
        &truncate_color,
        &conversion,
        verbose,
        sequence,
        duration,
        play_count,
    )?;

    Ok(())
}
