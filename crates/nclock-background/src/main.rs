use nclock_background::App;
use nclock_config::{AnimationConfig, AppConfig, Layer, LayerConfig};

fn main() {
    let config = AppConfig {
        animation: AnimationConfig {
            relative_inner_radius: 0.1,
            relative_lane_width: 0.045,
            relative_lane_margin: 0.015,
        },
        layer: LayerConfig {
            layer: Layer::Background,
            namespace: "nclock-background".to_string(),
            exit_on_input: false,
        },
    };

    App::run(config);
}
