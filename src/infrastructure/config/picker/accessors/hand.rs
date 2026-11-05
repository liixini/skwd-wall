use crate::contracts::picker::{HandAxis, HandCut, HandMove, HandVariance};

use super::super::Config;

impl Config {
    pub fn hand_position(&self) -> (f32, f32) {
        use skwd_config::schema::setting::selector as sel;
        (
            sel::HAND_STAGE_X.read(self.root()) as f32 / 100.0,
            sel::HAND_STAGE_Y.read(self.root()) as f32 / 100.0,
        )
    }

    pub fn hand_count(&self) -> usize {
        (self.num_at(skwd_config::keys::selector::HAND_COUNT, 5.0) as i64).clamp(2, 16) as usize
    }

    pub fn hand_fan_angle(&self) -> f32 {
        (self.num_at(skwd_config::keys::selector::HAND_FAN_ANGLE, 12.0) as f32).clamp(-60.0, 60.0)
    }

    pub fn hand_fan_roll(&self) -> f32 {
        (self.num_at(skwd_config::keys::selector::HAND_FAN_ROLL, 8.5) as f32).clamp(-45.0, 45.0)
    }

    pub fn hand_arch(&self) -> f32 {
        (self.num_at(skwd_config::keys::selector::HAND_ARCH, 20.0) as f32).clamp(-200.0, 200.0)
    }

    pub fn hand_skew(&self) -> f32 {
        (self.num_at(skwd_config::keys::selector::HAND_SKEW, 0.0) as f32).clamp(-200.0, 200.0)
    }

    pub fn hand_backdrop_blur(&self) -> f32 {
        (self.num_at(skwd_config::keys::selector::HAND_BACKDROP_BLUR, 100.0) as f32 / 100.0)
            .clamp(0.0, 4.0)
    }

    pub fn hand_card_width(&self) -> f32 {
        (self.num_at(skwd_config::keys::selector::HAND_CARD_WIDTH, 168.0) as f32).max(40.0)
    }

    pub fn hand_card_height(&self) -> f32 {
        (self.num_at(skwd_config::keys::selector::HAND_CARD_HEIGHT, 432.0) as f32).max(60.0)
    }

    pub fn hand_spread(&self) -> f32 {
        (self.num_at(skwd_config::keys::selector::HAND_SPREAD, 126.0) as f32).clamp(10.0, 600.0)
    }

    pub fn hand_ribbons(&self) -> usize {
        (self.num_at(skwd_config::keys::selector::HAND_RIBBONS, 6.0) as i64).clamp(2, 14) as usize
    }

    pub fn hand_speed(&self) -> f32 {
        (self.num_at(skwd_config::keys::selector::HAND_SPEED, 100.0) as f32 / 100.0)
            .clamp(0.35, 2.5)
    }

    pub fn hand_tilt(&self) -> f32 {
        (self.num_at(skwd_config::keys::selector::HAND_TILT, 100.0) as f32 / 100.0).clamp(0.0, 3.0)
    }

    pub fn hand_perspective(&self) -> f32 {
        (self.num_at(skwd_config::keys::selector::HAND_PERSPECTIVE, 1700.0) as f32)
            .clamp(400.0, 6000.0)
    }

    pub fn hand_ribbon_axis(&self) -> String {
        HandAxis::from_key(&self.str_path(skwd_config::keys::selector::HAND_RIBBON_AXIS))
            .as_key()
            .to_string()
    }

    pub fn hand_cut(&self) -> String {
        HandCut::from_key(&self.str_path(skwd_config::keys::selector::HAND_CUT))
            .as_key()
            .to_string()
    }

    pub fn hand_cut_variance(&self) -> String {
        HandVariance::from_key(&self.str_path(skwd_config::keys::selector::HAND_CUT_VARIANCE))
            .as_key()
            .to_string()
    }

    pub fn hand_move(&self) -> String {
        if self.str_path(skwd_config::keys::selector::HAND_MOVE) == "random" {
            String::from("random")
        } else {
            String::from("cycle")
        }
    }

    pub fn hand_moves(&self) -> [bool; 5] {
        if let Some(only) =
            HandMove::from_key(&self.str_path(skwd_config::keys::selector::HAND_MOVE))
        {
            let mut moves = [false; 5];
            moves[only.index()] = true;
            return moves;
        }
        [
            skwd_config::keys::selector::HAND_MOVE_CORKSCREW,
            skwd_config::keys::selector::HAND_MOVE_CASCADE,
            skwd_config::keys::selector::HAND_MOVE_SHUFFLE,
            skwd_config::keys::selector::HAND_MOVE_RIBBON,
            skwd_config::keys::selector::HAND_MOVE_SPIRAL,
        ]
        .map(|key| self.flag_default_config(key))
    }

    pub fn hand_ghosts(&self) -> bool {
        self.flag_default_config(skwd_config::keys::selector::HAND_GHOSTS)
    }

    pub fn hand_bob(&self) -> bool {
        self.flag_default_config(skwd_config::keys::selector::HAND_BOB)
    }

    pub fn hand_backdrop(&self) -> bool {
        self.flag_default_config(skwd_config::keys::selector::HAND_BACKDROP)
    }

    pub fn hand_reveal_fill(&self) -> bool {
        self.flag_default_config(skwd_config::keys::selector::HAND_REVEAL_FILL)
    }
}
