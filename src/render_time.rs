use std::time::Instant;

pub struct RenderTimeDiagnostic {
    start_time: Instant,
    prev_time: Instant,
    frame: usize,
}
impl RenderTimeDiagnostic {
    pub fn new() -> RenderTimeDiagnostic {
        RenderTimeDiagnostic {
            start_time: Instant::now(),
            prev_time: Instant::now(),
            frame: 0,
        }
    }
    pub fn increment(&mut self) -> RenderTime {
        self.frame += 1;
        let current = Instant::now();
        let delta = current - self.prev_time;
        self.prev_time = current;
        RenderTime(delta.as_secs_f32() * 1000.)
    }
    pub fn peak(&self) -> RenderTime {
        let current = Instant::now();
        let delta = current - self.prev_time;
        RenderTime(delta.as_secs_f32() * 1000.)
    }
    pub fn avg_render_time(&self) -> RenderTime {
        let delta = self.prev_time - self.start_time;
        let avg = delta.as_secs_f32() / self.frame as f32 * 1000.;
        RenderTime(avg)
    }
}
/// Render time in ms
pub struct RenderTime(pub f32);
