use std::time::Instant;

use log::info;

pub struct Timer {
    app_start: Instant,
    frame_starts: Vec<Instant>,
    frame_count: usize,
    log_frequency: usize,
}
impl Timer {
    pub fn new() -> Timer {
        Timer {
            app_start: Instant::now(),
            frame_starts: vec![Instant::now()],
            frame_count: 0,
            log_frequency: 2000,
        }
    }
    pub fn update(&mut self) {
        self.increment_frame();
        self.log();
    }
    pub fn dt(&self) -> f32 {
        let len = self.frame_starts.len();

        let dt = Instant::now().duration_since(self.frame_starts[len - 2]);
        dt.as_secs_f32()
    }
    fn increment_frame(&mut self) {
        self.frame_count += 1;
        self.frame_starts.push(Instant::now());
        if self.frame_starts.len() > self.log_frequency {
            self.frame_starts.remove(0);
        }
    }
    fn log(&self) {
        if self.frame_count % self.log_frequency != 0 {
            return;
        }
        let i_last = self.log_frequency - 1;
        let avg = self.frame_starts[i_last]
            .duration_since(self.frame_starts[0])
            .as_secs_f32()
            * 1000.
            / self.log_frequency as f32;
        let avg_total = self.frame_starts[i_last]
            .duration_since(self.app_start)
            .as_secs_f32()
            * 1000.
            / self.frame_count as f32;
        let current = self.frame_starts[i_last]
            .duration_since(self.frame_starts[i_last - 1])
            .as_secs_f32()
            * 1000.;
        info!(
            "avg {}: {:.2}ms, avg total: {:.2}ms, current: {:.2}ms",
            self.log_frequency, avg, avg_total, current
        );
    }
}
