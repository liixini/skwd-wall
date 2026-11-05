use serde_json::Value;

use super::super::Config;

impl Config {
    pub fn slice_shadows(&self) -> bool {
        skwd_config::schema::setting::selector::SLICE_SHADOWS.read(self.root())
    }

    pub fn depth_shadows(&self) -> bool {
        skwd_config::schema::setting::selector::DEPTH_SHADOWS.read(self.root())
    }

    pub fn depth_selection_frame(&self) -> bool {
        skwd_config::schema::setting::selector::DEPTH_SELECTION_FRAME.read(self.root())
    }
    pub fn depth_height(&self) -> f32 {
        skwd_config::schema::setting::selector::DEPTH_HEIGHT.read(self.root()) as f32
    }
    pub fn depth_width(&self) -> f32 {
        skwd_config::schema::setting::selector::DEPTH_WIDTH_PX.read(self.root()) as f32
    }
    pub fn depth_spacing(&self) -> f32 {
        skwd_config::schema::setting::selector::DEPTH_SPACING_PX.read(self.root()) as f32
    }
    pub fn depth_count(&self) -> f32 {
        skwd_config::schema::setting::selector::DEPTH_COUNT.read(self.root()) as f32
    }
    pub fn depth_falloff(&self) -> f32 {
        skwd_config::schema::setting::selector::DEPTH_FALLOFF_FACTOR.read(self.root()) as f32
    }
    pub fn depth_skew(&self) -> f32 {
        skwd_config::schema::setting::selector::DEPTH_SKEW.read(self.root()) as f32
    }
    pub fn depth_navigation_ms(&self) -> f32 {
        skwd_config::schema::setting::selector::DEPTH_NAVIGATION_MS.read(self.root()) as f32
    }
    pub fn collection_size(&self) -> f32 {
        skwd_config::schema::setting::selector::COLLECTION_SIZE.read(self.root()) as f32
    }
    pub fn collection_spacing(&self) -> f32 {
        skwd_config::schema::setting::selector::COLLECTION_SPACING.read(self.root()) as f32
    }
    pub fn collection_count(&self) -> f32 {
        skwd_config::schema::setting::selector::COLLECTION_COUNT.read(self.root()) as f32
    }
    pub fn collection_tilt(&self) -> f32 {
        skwd_config::schema::setting::selector::COLLECTION_TILT.read(self.root()) as f32
    }
    pub fn collection_speed(&self) -> f32 {
        skwd_config::schema::setting::selector::COLLECTION_SPEED.read(self.root()) as f32
    }
    pub fn slice_position(&self) -> (f32, f32) {
        use skwd_config::schema::setting::selector as sel;
        (
            sel::SLICE_STAGE_X.read(self.root()) as f32 / 100.0,
            sel::SLICE_STAGE_Y.read(self.root()) as f32 / 100.0,
        )
    }

    pub fn sandy_position(&self) -> (f32, f32) {
        use skwd_config::schema::setting::selector as sel;
        (
            sel::SANDY_STAGE_X.read(self.root()) as f32 / 100.0,
            sel::SANDY_STAGE_Y.read(self.root()) as f32 / 100.0,
        )
    }

    pub fn tag_cloud_offset(&self) -> (f32, f32) {
        (
            self.sel_num(skwd_config::keys::selector::TAG_CLOUD_OFFSET_X, 0.0, 0.0) as f32,
            self.sel_num(skwd_config::keys::selector::TAG_CLOUD_OFFSET_Y, 0.0, 0.0) as f32,
        )
    }

    pub fn tag_cloud_position(&self) -> Option<(f32, f32)> {
        use skwd_config::schema::setting::selector as sel;

        self.plate_position(
            &sel::TAG_CLOUD_POSITION_ENABLED,
            &sel::TAG_CLOUD_POSITION_X,
            &sel::TAG_CLOUD_POSITION_Y,
        )
    }

    pub fn filter_swap_ms(&self) -> f32 {
        self.filter_swap_motion_ms()
    }

    skwd_config::getters! {
        slice_height: sel_num(skwd_config::keys::selector::SLICE_HEIGHT, 520.0, 360.0) as f32;
        visible_count: sel_num(skwd_config::keys::selector::VISIBLE_COUNT, 12.0, 8.0) as usize;
        expanded_width: sel_num(skwd_config::keys::selector::EXPANDED_WIDTH, 924.0, 600.0) as f32;
        slice_width: sel_num(skwd_config::keys::selector::SLICE_WIDTH, 135.0, 90.0) as f32;
        slice_spacing: sel_num(skwd_config::keys::selector::SLICE_SPACING, 0.0, 0.0) as f32;
        skew_offset: sel_num(skwd_config::keys::selector::SKEW_OFFSET, 35.0, 25.0) as f32;
        slice_edge_tilt: sel_num(skwd_config::keys::selector::SLICE_EDGE_TILT, 0.0, 0.0) as f32;
        grid_thumb_width: sel_num(skwd_config::keys::selector::GRID_THUMB_WIDTH, 300.0, 220.0) as f32;
        grid_thumb_height: sel_num(skwd_config::keys::selector::GRID_THUMB_HEIGHT, 169.0, 124.0) as f32;
        hex_radius: sel_num(skwd_config::keys::selector::HEX_RADIUS, 140.0, 100.0) as f32;
    }

    pub fn corners(
        &self,
        keys: &skwd_config::corners::CornerKeys,
    ) -> skwd_config::corners::Corners {
        keys.read(self.root())
    }

    pub fn stored(&self, path: &str) -> Option<serde_json::Value> {
        self.get(path).cloned()
    }

    pub fn corners_all_stored(&self, keys: &skwd_config::corners::CornerKeys) -> bool {
        self.get(keys.all).is_some()
    }

    pub fn tag_cloud_width(&self) -> f32 {
        self.sel_num(skwd_config::keys::selector::TAG_CLOUD_WIDTH, 760.0, 540.0)
            .clamp(360.0, 2200.0) as f32
    }

    pub fn tag_cloud_width_override(&self) -> Option<f64> {
        self.get(skwd_config::keys::selector::TAG_CLOUD_WIDTH).and_then(Value::as_f64)
    }

    pub fn tag_cloud_height(&self) -> f32 {
        self.sel_num(skwd_config::keys::selector::TAG_CLOUD_HEIGHT, 168.0, 150.0).clamp(90.0, 600.0)
            as f32
    }

    pub fn tag_cloud_rows(&self) -> usize {
        match self.get(skwd_config::keys::selector::TAG_CLOUD_ROWS) {
            Some(Value::String(value)) => value.parse().unwrap_or(2),
            Some(Value::Number(value)) => value.as_u64().unwrap_or(2) as usize,
            _ => 2,
        }
        .clamp(1, 3)
    }

    pub fn grid_columns(&self) -> usize {
        (self.sel_num(skwd_config::keys::selector::GRID_COLUMNS, 6.0, 4.0) as usize).max(1)
    }

    pub fn grid_rows(&self) -> usize {
        (self.sel_num(skwd_config::keys::selector::GRID_ROWS, 3.0, 3.0) as usize).max(1)
    }

    pub fn grid_gap_x(&self) -> f32 {
        skwd_config::schema::setting::selector::GRID_GAP_X.read(self.root()) as f32
    }

    pub fn grid_gap_y(&self) -> f32 {
        skwd_config::schema::setting::selector::GRID_GAP_Y.read(self.root()) as f32
    }

    pub fn grid_border_width(&self) -> f32 {
        skwd_config::schema::setting::selector::GRID_BORDER_WIDTH.read(self.root()) as f32
    }

    pub fn hex_rows(&self) -> usize {
        (self.sel_num(skwd_config::keys::selector::HEX_ROWS, 3.0, 3.0) as usize).max(1)
    }

    pub fn hex_cols(&self) -> usize {
        (self.sel_num(skwd_config::keys::selector::HEX_COLS, 7.0, 5.0) as usize).max(3)
    }

    pub fn hex_scroll_step(&self) -> usize {
        (self.sel_num(skwd_config::keys::selector::HEX_SCROLL_STEP, 1.0, 1.0) as usize).max(1)
    }

    pub fn hex_arc_intensity(&self) -> f32 {
        self.num_at(skwd_config::keys::selector::HEX_ARC_INTENSITY, 1.2) as f32
    }

    pub fn slice_wobble_strength(&self) -> f32 {
        (self.sel_num(skwd_config::keys::selector::SLICE_WOBBLE_STRENGTH, 100.0, 100.0) as f32
            / 100.0)
            .clamp(0.0, 2.0)
    }
}
