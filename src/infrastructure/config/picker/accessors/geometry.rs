use serde_json::Value;

use super::super::Config;

const PERCENT: f32 = 100.0;

impl Config {
    pub fn picker_frame(&self) -> Option<[f32; 4]> {
        use skwd_config::schema::setting::selector as sel;

        if !sel::FRAME_ENABLED.read(self.root()) {
            return None;
        }
        let width = sel::FRAME_WIDTH.read(self.root()) as f32 / PERCENT;
        let height = sel::FRAME_HEIGHT.read(self.root()) as f32 / PERCENT;
        let x = (sel::FRAME_X.read(self.root()) as f32 / PERCENT).min(1.0 - width);
        let y = (sel::FRAME_Y.read(self.root()) as f32 / PERCENT).min(1.0 - height);
        Some([x, y, width, height])
    }

    pub fn grid_layout(&self) -> String {
        let value = skwd_config::schema::setting::selector::GRID_LAYOUT.read(self.root());
        match value.as_str() {
            "uniform" | "brick" | "masonry" | "justified" | "editorial" | "cylinder" => value,
            _ => String::from("uniform"),
        }
    }

    pub fn grid_stagger(&self) -> f32 {
        skwd_config::schema::setting::selector::GRID_STAGGER.read(self.root()) as f32 / 100.0
    }

    pub fn grid_selected_scale(&self) -> f32 {
        skwd_config::schema::setting::selector::GRID_SELECTED_SCALE.read(self.root()) as f32 / 100.0
    }

    pub fn grid_flow_wave(&self) -> f32 {
        skwd_config::schema::setting::selector::GRID_FLOW_WAVE.read(self.root()) as f32
    }

    pub fn grid_flow_frequency(&self) -> f32 {
        skwd_config::schema::setting::selector::GRID_FLOW_FREQUENCY.read(self.root()) as f32
    }

    pub fn grid_scatter(&self) -> f32 {
        skwd_config::schema::setting::selector::GRID_SCATTER.read(self.root()) as f32
    }

    pub fn grid_scale_variance(&self) -> f32 {
        skwd_config::schema::setting::selector::GRID_SCALE_VARIANCE.read(self.root()) as f32 / 100.0
    }

    pub fn grid_cylinder_bend(&self) -> f32 {
        skwd_config::schema::setting::selector::GRID_CYLINDER_BEND.read(self.root()) as f32 / 100.0
    }

    pub fn grid_cylinder_radius(&self) -> f32 {
        skwd_config::schema::setting::selector::GRID_CYLINDER_RADIUS.read(self.root()) as f32
    }

    pub fn grid_stage(&self) -> [f32; 8] {
        use skwd_config::schema::setting::selector as sel;
        [
            sel::GRID_STAGE_X.read(self.root()) as f32 / 100.0,
            sel::GRID_STAGE_Y.read(self.root()) as f32 / 100.0,
            sel::GRID_STAGE_SCALE.read(self.root()) as f32 / 100.0,
            sel::GRID_STAGE_ROTATION.read(self.root()) as f32,
            sel::GRID_STAGE_PERSPECTIVE.read(self.root()) as f32 / 100.0,
            sel::GRID_STAGE_SHEAR_X.read(self.root()) as f32 / 100.0,
            sel::GRID_STAGE_SHEAR_Y.read(self.root()) as f32 / 100.0,
            sel::GRID_STAGE_DEPTH_ANGLE.read(self.root()) as f32,
        ]
    }

    pub fn hex_curve(&self) -> String {
        let explicit = self.get(skwd_config::keys::selector::HEX_CURVE).and_then(Value::as_str);
        if explicit.is_none() && !self.hex_arc() {
            return String::from("flat");
        }
        match skwd_config::schema::setting::selector::HEX_CURVE.read(self.root()).as_str() {
            "flat" => String::from("flat"),
            "wave" => String::from("wave"),
            "s" => String::from("s"),
            "cylinder" => String::from("cylinder"),
            _ => String::from("arc"),
        }
    }

    pub fn hex_curve_frequency(&self) -> f32 {
        skwd_config::schema::setting::selector::HEX_CURVE_FREQUENCY.read(self.root()) as f32
    }

    pub fn hex_shape(&self) -> String {
        match skwd_config::schema::setting::selector::HEX_SHAPE.read(self.root()).as_str() {
            "triangle" => String::from("triangle"),
            "diamond" => String::from("diamond"),
            "rhombus" => String::from("rhombus"),
            _ => String::from("hexagon"),
        }
    }

    pub fn hex_gap_x(&self) -> f32 {
        skwd_config::schema::setting::selector::HEX_GAP_X.read(self.root()) as f32
    }

    pub fn hex_gap_y(&self) -> f32 {
        skwd_config::schema::setting::selector::HEX_GAP_Y.read(self.root()) as f32
    }

    pub fn hex_aspect(&self) -> f32 {
        skwd_config::schema::setting::selector::HEX_ASPECT.read(self.root()) as f32 / 100.0
    }

    pub fn hex_stagger(&self) -> f32 {
        skwd_config::schema::setting::selector::HEX_STAGGER.read(self.root()) as f32 / 100.0
    }

    pub fn hex_lens(&self) -> f32 {
        skwd_config::schema::setting::selector::HEX_LENS.read(self.root()) as f32 / 100.0
    }

    pub fn hex_lens_radius(&self) -> f32 {
        skwd_config::schema::setting::selector::HEX_LENS_RADIUS.read(self.root()) as f32
    }

    pub fn hex_orbit(&self) -> f32 {
        skwd_config::schema::setting::selector::HEX_ORBIT.read(self.root()) as f32 / 100.0
    }

    pub fn hex_orbit_radius(&self) -> f32 {
        skwd_config::schema::setting::selector::HEX_ORBIT_RADIUS.read(self.root()) as f32
    }

    pub fn hex_twist(&self) -> f32 {
        skwd_config::schema::setting::selector::HEX_TWIST.read(self.root()) as f32
    }

    pub fn hex_scatter(&self) -> f32 {
        skwd_config::schema::setting::selector::HEX_SCATTER.read(self.root()) as f32
    }

    pub fn hex_stage(&self) -> [f32; 8] {
        use skwd_config::schema::setting::selector as sel;
        [
            sel::HEX_STAGE_X.read(self.root()) as f32 / 100.0,
            sel::HEX_STAGE_Y.read(self.root()) as f32 / 100.0,
            sel::HEX_STAGE_SCALE.read(self.root()) as f32 / 100.0,
            sel::HEX_STAGE_ROTATION.read(self.root()) as f32,
            sel::HEX_STAGE_PERSPECTIVE.read(self.root()) as f32 / 100.0,
            sel::HEX_STAGE_SHEAR_X.read(self.root()) as f32 / 100.0,
            sel::HEX_STAGE_SHEAR_Y.read(self.root()) as f32 / 100.0,
            sel::HEX_STAGE_DEPTH_ANGLE.read(self.root()) as f32,
        ]
    }
}
