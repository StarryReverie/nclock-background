use clap::Parser;
use nclock_background::App;
use nclock_config::{AnimationConfig, AppConfig, Layer, LayerConfig};

fn main() {
    let cli = Cli::parse();

    let config = AppConfig {
        animation: AnimationConfig {
            relative_inner_radius: cli.relative_inner_radius,
            relative_lane_width: cli.relative_lane_width,
            relative_lane_margin: cli.relative_lane_margin,
            hue_start: cli.hue_start,
            hue_end: cli.hue_end,
        },
        layer: LayerConfig {
            layer: cli.layer,
            namespace: cli.namespace,
            exit_on_input: cli.exit_on_input,
        },
    };

    App::run(config);
}

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Fancy dynamic night clock wallpaper engine for Wayland compositors"
)]
struct Cli {
    /// Inner radius of the innermost orbit, as a fraction of the display height.
    #[arg(long, default_value = "0.1")]
    relative_inner_radius: f64,

    /// Width of each orbit lane, as a fraction of the display height.
    #[arg(long, default_value = "0.045")]
    relative_lane_width: f64,

    /// Margin between adjacent orbit lanes, as a fraction of the display height.
    #[arg(long, default_value = "0.015")]
    relative_lane_margin: f64,

    /// Start of the hue interval mapped to the pointer angle (0.0 = red).
    ///
    /// The pointer angle sweeps linearly from `hue_start` to `hue_end` around the HSL color wheel.
    /// Values wrap freely, so `hue_start > hue_end` produces a reverse sweep and equal values
    /// produce a solid color.
    #[arg(long, default_value = "0.0")]
    hue_start: f32,

    /// End of the hue interval mapped to the pointer angle (1.0 = red).
    #[arg(long, default_value = "1.0")]
    hue_end: f32,

    /// Which layer shell layer to use.
    ///
    /// Supported values: `background`, `bottom`, `top`, `overlay`.
    #[arg(long, default_value = "background", value_parser = parse_layer)]
    layer: Layer,

    /// Namespace string sent to the compositor with the layer surface.
    #[arg(long, default_value = "nclock-background")]
    namespace: String,

    /// Exit the program when a key is pressed or mouse is clicked.
    #[arg(long, default_value_t = false)]
    exit_on_input: bool,
}

fn parse_layer(s: &str) -> Result<Layer, String> {
    match s.to_ascii_lowercase().as_str() {
        "background" => Ok(Layer::Background),
        "bottom" => Ok(Layer::Bottom),
        "top" => Ok(Layer::Top),
        "overlay" => Ok(Layer::Overlay),
        _ => Err(format!(
            "invalid layer '{}': expected one of background, bottom, top, overlay",
            s
        )),
    }
}
