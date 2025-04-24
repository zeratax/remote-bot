pub struct GameConstants {
    pub minimum_options: &'static [u32],
    pub interest_options: &'static [f64],
    pub penalty_options: &'static [f64],
}

pub static MINIMUM_OPTIONS: [u32; 5] = [10, 20, 30, 100, 200];
pub static INTEREST_OPTIONS: [f64; 5] = [0.02, 0.05, 0.08, 0.10, 0.20];
pub static PENALTY_OPTIONS: [f64; 6] = [0.05, 0.08, 0.12, 0.20, 0.30, 0.50];

pub static GAME_CONSTANTS: GameConstants = GameConstants {
    minimum_options: &MINIMUM_OPTIONS,
    interest_options: &INTEREST_OPTIONS,
    penalty_options: &PENALTY_OPTIONS,
};
