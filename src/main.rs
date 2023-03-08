use anyhow::Context;
use anyhow::Result;
use skia_safe::image_filters::drop_shadow_only;
use skia_safe::EncodedImageFormat;
use skia_safe::Surface;
use skia_safe::{Color, Data, IRect, Image, Point, RRect, Rect};

use std::io::{self, Read, Write};
use std::fs::{self, File};

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
        let r: u8 = u8::from_str_radix(&value[2..4],  16).unwrap_or(0);
        let g: u8 = u8::from_str_radix(&value[4..6],  16).unwrap_or(0);
        let b: u8 = u8::from_str_radix(&value[6..8],  16).unwrap_or(0);
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
#[command(about = "Adds shadow, padding and rounded corners to images", long_about = None)]
struct Args {
    #[arg(short = 'r', long, default_value_t = 12.0, help = "border radius")]
    radius: f32,
    #[arg(short = 'p', long, default_value_t = 1./6., help = "padding = max((img.width * padding_ratio), (img.height * padding.ratio))")]
    padding_ratio: f32,
    #[arg(short = 'b', long, default_value_t = 1./6., help = "blur = padding * blur_ratio")]
    blur_ratio: f32,
    #[arg(short = 'c', long, default_value_t = String::from("0x00000064"), help = "0xrrggbbaa")]
    shadow_color: String,
    #[arg(short = 'x', long, default_value_t = 0., help = "offset_x = padding * offsetx_ratio")]
    offsetx_ratio: f32,
    #[arg(short = 'y', long, default_value_t = 1./8., help = "offset_y = padding * offsety_ratio")]
    offsety_ratio: f32,
    #[arg(short = 'i', long, default_value_t = String::from("-"), help = "path to input file / - for stdin")]
    input: String,
    #[arg(short = 'o', long, default_value_t = String::from("-"), help = "path to output file / - for stdout")]
    output: String,
}

fn main() -> Result<()> {

    let args = Args::parse();

    let shadow_color: Color = ShadowColor::from(args.shadow_color).into();

    let mut user_input: Vec<u8> = Vec::new();
    match args.input.as_str() {
        "-" => {
            io::stdin()
                .read_to_end(&mut user_input)
                .context("Failed to read from stdin")?;
        },
        path => {
            user_input = fs::read(path)
                .context(format!("Failed to read file {}", path))?;
        }
    };

    let img: Image = unsafe {
        Image::from_encoded(Data::new_bytes(&user_input)).context("Failed to decode image")?
    };

    let x_padding = (img.width() as f32 * args.padding_ratio) as i32;
    let y_padding = (img.height() as f32 * args.padding_ratio) as i32;

    let padding = x_padding.max(y_padding);

    let mut canvas = new_canvas(img.width() + padding, img.height() + padding);

    canvas.canvas().clip_rrect(
        RRect::new_rect_radii(
            Rect {
                left: (padding / 2) as f32,
                top: (padding / 2) as f32,
                bottom: (img.height() + padding / 2) as f32,
                right: (img.width() + padding / 2) as f32,
            },
            &[
                Point::new(args.radius, args.radius),
                Point::new(args.radius, args.radius),
                Point::new(args.radius, args.radius),
                Point::new(args.radius, args.radius),
            ],
        ),
        None,
        true,
    );

    canvas
        .canvas()
        .draw_image(img, (padding / 2, padding / 2), None);

    let padded_image = canvas.image_snapshot();

    let filter = drop_shadow_only(
        (padding as f32 * args.offsetx_ratio, padding as f32 * args.offsety_ratio),
        (padding as f32 * args.blur_ratio, padding as f32 * args.blur_ratio),
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
        },
        path => {
            let mut output = File::create(path)?;
            output
                .write_all(bytes)
                .context(format!("Failed to write to {}", path))?;
        }
    }

    Ok(())
}
