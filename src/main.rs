mod config;
mod math;

use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use clap::CommandFactory;
use config::ConfigFile;
use math::parse_math;
use skia_safe::image_filters::drop_shadow_only;
use skia_safe::EncodedImageFormat;
use skia_safe::Surface;
use skia_safe::{Color, Data, IRect, Image, Point, RRect, Rect};

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use clap::Parser;

fn new_canvas(width: i32, height: i32) -> Surface {
    let mut surface = Surface::new_raster_n32_premul((width, height)).expect("no surface!");
    surface.canvas().clear(Color::TRANSPARENT);
    surface
}

struct ShadowColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl From<String> for ShadowColor {
    fn from(value: String) -> Self {
        let r: u8 = u8::from_str_radix(&value[2..4], 16).unwrap_or(0);
        let g: u8 = u8::from_str_radix(&value[4..6], 16).unwrap_or(0);
        let b: u8 = u8::from_str_radix(&value[6..8], 16).unwrap_or(0);
        let a: u8 = u8::from_str_radix(&value[8..10], 16).unwrap_or(0);

        Self { r, g, b, a }
    }
}

impl Into<Color> for ShadowColor {
    fn into(self) -> Color {
        Color::from_argb(self.a, self.r, self.g, self.b)
    }
}

#[derive(Parser, Debug)]
#[command(name = "shadower")]
#[command(
    about = "Adds shadow, padding, and rounded corners to images.",
    long_about = None,
    after_help = "
    \x1b[1;4mMath Expressions\x1b[0m\nFor every numerical option here, you can use math expressions to calculate the values depending on the input image size.
    You can use the \x1b[1mwidth\x1b[0m, \x1b[1mheight\x1b[0m, \x1b[1mmax\x1b[0m, and \x1b[1mmin\x1b[0m keywords within the expressions.
    The expressions support addition, subtraction, multiplication and division.
    "
)]
struct Args {
    #[arg(
        short = 'r',
        long,
        default_value_t = String::from("8 + max / 100"),
        help = "border radius"
    )]
    radius: String,

    #[arg(
        long,
        default_value_t = String::from("max / 12 + 10"),
        help = "horizontal padding"
    )]
    padding_x: String,
    #[arg(
        long,
        default_value_t = String::from("max / 12 + 10"),
        help = "vertical padding"
    )]
    padding_y: String,

    #[arg(
        long,
        default_value_t = String::from("max / 36 + 5"),
        help = "horizontal shadow blur"
    )]
    blur_x: String,
    #[arg(
        long,
        default_value_t = String::from("max / 36 + 5"),
        help = "vertical shadow blur"
    )]
    blur_y: String,

    #[arg(short = 'c', long, default_value_t = String::from("0x00000064"), help = "0xRRGGBBAA")]
    shadow_color: String,

    #[arg(
        long,
        default_value_t = String::from("0"),
        help = "horizontal shadow offset"
    )]
    offset_x: String,
    #[arg(
        long,
        default_value_t = String::from("height / 48"),
        help = "vertical shadow offset"
    )]
    offset_y: String,

    #[arg(short = 'i', long, default_value_t = String::from("-"), help = "path to input file / - for stdin")]
    input: String,
    #[arg(short = 'o', long, default_value_t = String::from("-"), help = "path to output file / - for stdout")]
    output: String,

    #[arg(long, default_value_t = String::from("$XDG_CONFIG_HOME/shadower/config.toml"), help = "path to config file")]
    config: String,
}

macro_rules! replace_default {
    ($args: ident, $config: ident, $matches: ident, $prop: ident) => {
        if let Some(clap::parser::ValueSource::DefaultValue) =
            $matches.value_source(stringify!($prop))
        {
            if let Some(v) = $config.$prop {
                $args.$prop = envmnt::expand(&v, None);
            }
        }
    };
}

fn main() -> Result<()> {
    let mut args = Args::parse();

    let matches = Args::command().get_matches();

    let mut is_default_config = true;

    let config_path = match matches.value_source("config") {
        None => dirs::config_dir().map(|mut dir| {
            dir.push("shadower/config.toml");
            dir
        }),
        Some(source) => match source {
            clap::parser::ValueSource::DefaultValue => dirs::config_dir().map(|mut dir| {
                dir.push("shadower/config.toml");
                dir
            }),
            _ => {
                is_default_config = false;
                let path = PathBuf::from(args.config.clone());

                std::fs::canonicalize(path).ok()
            }
        },
    };

    if let Some(config_path) = config_path {
        let config = ConfigFile::read(config_path);

        match config {
            Ok(config) => {
                replace_default!(args, config, matches, radius);

                replace_default!(args, config, matches, padding_x);
                replace_default!(args, config, matches, padding_y);

                replace_default!(args, config, matches, blur_x);
                replace_default!(args, config, matches, blur_y);

                replace_default!(args, config, matches, shadow_color);

                replace_default!(args, config, matches, offset_x);
                replace_default!(args, config, matches, offset_y);

                replace_default!(args, config, matches, input);
                replace_default!(args, config, matches, output);
            }
            Err(error) => match error {
                config::ConfigError::DeserializeFailed(err) => bail!(err),
                config::ConfigError::MissingFile(err) => {
                    if !is_default_config {
                        bail!(err);
                    }
                }
            },
        }
    }

    let shadow_color: Color = ShadowColor::from(args.shadow_color).into();

    let mut user_input: Vec<u8> = Vec::new();
    match args.input.as_str() {
        "-" => {
            io::stdin()
                .read_to_end(&mut user_input)
                .context("Failed to read from stdin")?;
        }
        path => {
            user_input = fs::read(path).context(format!("Failed to read file {}", path))?;
        }
    };

    let img: Image = unsafe {
        Image::from_encoded(Data::new_bytes(&user_input)).context("Failed to decode image")?
    };

    let orig_width = img.width();
    let orig_height = img.height();

    let x_padding = parse_math(args.padding_x, orig_width, orig_height) as i32;
    let y_padding = parse_math(args.padding_y, orig_width, orig_height) as i32;

    let mut canvas = new_canvas(orig_width + x_padding * 2, orig_height + y_padding * 2);

    let radius = parse_math(args.radius, orig_width, orig_height);
    let point = Point::new(radius, radius);

    canvas.canvas().clip_rrect(
        RRect::new_rect_radii(
            Rect {
                left: x_padding as f32,
                top: y_padding as f32,
                bottom: (img.height() + y_padding) as f32,
                right: (img.width() + x_padding) as f32,
            },
            &[point.clone(), point.clone(), point.clone(), point],
        ),
        None,
        true,
    );

    canvas
        .canvas()
        .draw_image(img, (x_padding, y_padding), None);

    let padded_image = canvas.image_snapshot();

    let filter = drop_shadow_only(
        (
            parse_math(args.offset_x, orig_width, orig_height),
            parse_math(args.offset_y, orig_width, orig_height),
        ),
        (
            parse_math(args.blur_x, orig_width, orig_height),
            parse_math(args.blur_y, orig_width, orig_height),
        ),
        shadow_color,
        None,
        None,
    )
    .context("Failed to create drop shadow")?;

    let shadow_image = padded_image
        .new_with_filter(
            &filter,
            IRect {
                left: 0,
                top: 0,
                right: padded_image.width(),
                bottom: padded_image.height(),
            },
            IRect {
                left: 0,
                top: 0,
                right: padded_image.width(),
                bottom: padded_image.height(),
            },
        )
        .context("Failed to add drop shadow")?
        .0;

    let mut canvas = new_canvas(padded_image.width(), padded_image.height());

    canvas.canvas().draw_image(shadow_image, (0, 0), None);

    canvas.canvas().draw_image(padded_image, (0, 0), None);

    let image = canvas.image_snapshot();
    let data = image
        .encode_to_data(EncodedImageFormat::PNG)
        .context("Failed to encode image")?;

    let bytes = data.as_bytes();
    match args.output.as_str() {
        "-" => {
            io::stdout()
                .write_all(bytes)
                .context("Failed to write to stdout")?;
        }
        path => {
            let mut output = File::create(path)?;
            output
                .write_all(bytes)
                .context(format!("Failed to write to {}", path))?;
        }
    }

    Ok(())
}
