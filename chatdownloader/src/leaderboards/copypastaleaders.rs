/*
Copypasta leaders leaderboard
*/

use crate::_types::clptypes::{ChatPerformance, PerformanceType};
use crate::_types::leaderboardtypes::LeaderboardInnerState;
use crate::leaderboards::leaderboardtrait::AbstractLeaderboard;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct CopypastaLeaders {
    state: HashMap<String, LeaderboardInnerState>,
}

impl AbstractLeaderboard for CopypastaLeaders {
    fn new() -> Self {
        let mut out = Self {
            state: HashMap::new(),
        };
        out.read_initial_state();
        out
    }

    fn get_name(&self) -> String {
        "copypasta".to_string()
    }

    fn __get_state(&mut self) -> &mut HashMap<String, LeaderboardInnerState> {
        &mut self.state
    }

    fn __performance_type(&self) -> PerformanceType {
        PerformanceType::User
    }

    fn calculate_score(&self, performance: &ChatPerformance) -> Option<f32> {
        Some(*performance.metrics.get("copypasta").unwrap_or(&0.0))
    }
}
