/*
The overall leaderboard
*/

use crate::_types::clptypes::{ChatPerformance, PerformanceType};
use crate::_types::leaderboardtypes::LeaderboardInnerState;
use crate::leaderboards::leaderboardtrait::AbstractLeaderboard;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct Overall {
    state: HashMap<String, LeaderboardInnerState>,
}

impl AbstractLeaderboard for Overall {
    fn new() -> Self {
        let mut out = Self {
            state: HashMap::new(),
        };
        out.read_initial_state();
        out
    }

    fn get_name(&self) -> String {
        "overall".to_string()
    }

    fn __get_state(&mut self) -> &mut HashMap<String, LeaderboardInnerState> {
        &mut self.state
    }

    fn __performance_type(&self) -> PerformanceType {
        PerformanceType::User
    }

    fn calculate_score(&self, performance: &ChatPerformance) -> Option<f32> {
        return Some(performance.metrics.values().sum());
    }
}
