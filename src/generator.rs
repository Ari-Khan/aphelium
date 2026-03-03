pub struct StrategyParams {
    pub buy_lb: i32,
    pub buy_thr: f64,
    pub sell_lb: i32,
    pub sell_thr: f64,
    pub max_slots: usize,
    pub exit_days: i32,
}

pub fn create_search_space() -> Vec<StrategyParams> {
    let mut space = Vec::new();
    
    let thresholds = [-0.05, -0.02, -0.01, 0.01, 0.02, 0.05];
    let lookbacks = [1, 5, 21, 73];
    let slots = [1, 2, 3];
    let exit_timers = [0, 5, 21];

    for &b_t in &thresholds {
        for &s_t in &thresholds {
            for &b_lb in &lookbacks {
                for &s_lb in &lookbacks {
                    for &sl in &slots {
                        for &et in &exit_timers {
                            space.push(StrategyParams {
                                buy_lb: b_lb,
                                buy_thr: b_t,
                                sell_lb: s_lb,
                                sell_thr: s_t,
                                max_slots: sl,
                                exit_days: et,
                            });
                        }
                    }
                }
            }
        }
    }
    space
}