// ============================================
// TIMING UTILITY - Performance Measurement
// ============================================
// Usage:
//   1. As a wrapper: let result = timed("operation_name", || { /* code */ });
//   2. As async wrapper: let result = timed_async("operation_name", async { /* code */ }).await;
//   3. Manual tracking: let timer = Timer::start("name"); ... timer.stop();
//   4. Section tracking: Timer::section("name", || { /* code */ });
// ============================================

use std::time::{Duration, Instant};
use colored::Colorize;

/// Timer for measuring execution time
pub struct Timer {
    name: String,
    start: Instant,
    threshold_ms: u128,
    silent: bool,
}

impl Timer {
    /// Create a new timer with a name
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            threshold_ms: 0,
            silent: false,
        }
    }

    /// Create a timer that only logs if execution exceeds threshold (in milliseconds)
    pub fn start_with_threshold(name: impl Into<String>, threshold_ms: u128) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            threshold_ms,
            silent: false,
        }
    }

    /// Create a silent timer (won't auto-log on drop, use elapsed() manually)
    pub fn silent(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            threshold_ms: 0,
            silent: true,
        }
    }

    /// Get elapsed time without stopping the timer
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Stop the timer and log the result
    pub fn stop(self) -> Duration {
        let duration = self.start.elapsed();
        self.log_duration(duration);
        duration
    }

    /// Log duration with emoji based on time taken
    fn log_duration(&self, duration: Duration) {
        if self.silent {
            return;
        }

        let ms = duration.as_millis();
        
        // Only log if above threshold
        if ms < self.threshold_ms {
            return;
        }

        let (emoji, _color) = Self::get_emoji_and_color(ms);
        
        if ms < 1000 {
            println!("{} {} - {}ms", emoji, self.name.as_str().cyan(), ms);
        } else {
            println!("{} {} - {:.2}s", emoji, self.name.as_str().cyan(), duration.as_secs_f64());
        }
    }

    /// Get emoji and color based on duration
    fn get_emoji_and_color(ms: u128) -> (&'static str, &'static str) {
        match ms {
            0..=100 => ("⚡", "green"),        // Very fast
            101..=500 => ("✅", "green"),      // Fast
            501..=1000 => ("⏱️", "yellow"),    // Acceptable
            1001..=5000 => ("🐌", "yellow"),   // Slow
            _ => ("🔥", "red"),                 // Very slow
        }
    }

    // ============================================
    // WRAPPER FUNCTIONS FOR EASY TIMING
    // ============================================

    /// Time a synchronous closure
    pub fn measure<F, R>(name: impl Into<String>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let timer = Self::start(name);
        let result = f();
        timer.stop();
        result
    }

    /// Time a synchronous closure with threshold
    pub fn measure_if_slow<F, R>(name: impl Into<String>, threshold_ms: u128, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let timer = Self::start_with_threshold(name, threshold_ms);
        let result = f();
        timer.stop();
        result
    }

    /// Time an async function
    pub async fn measure_async<F, Fut, R>(name: impl Into<String>, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let timer = Self::start(name);
        let result = f().await;
        timer.stop();
        result
    }

    /// Time an async function with threshold
    pub async fn measure_async_if_slow<F, Fut, R>(
        name: impl Into<String>,
        threshold_ms: u128,
        f: F,
    ) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let timer = Self::start_with_threshold(name, threshold_ms);
        let result = f().await;
        timer.stop();
        result
    }

    /// Log a section header for grouping timings
    pub fn section(name: impl Into<String>) {
        let name = name.into();
        println!("\n{}", "=".repeat(60).blue());
        println!("{} {}", "⏱️".to_string(), name.cyan().bold());
        println!("{}", "=".repeat(60).blue());
    }
}

// Auto-log on drop if not silent
impl Drop for Timer {
    fn drop(&mut self) {
        if !self.silent {
            let duration = self.start.elapsed();
            self.log_duration(duration);
        }
    }
}

// ============================================
// CONVENIENCE FUNCTIONS
// ============================================

/// Time a synchronous closure (shorthand)
pub fn timed<F, R>(name: impl Into<String>, f: F) -> R
where
    F: FnOnce() -> R,
{
    Timer::measure(name, f)
}

/// Time a synchronous closure with threshold (shorthand)
pub fn timed_if_slow<F, R>(name: impl Into<String>, threshold_ms: u128, f: F) -> R
where
    F: FnOnce() -> R,
{
    Timer::measure_if_slow(name, threshold_ms, f)
}

/// Time an async function (shorthand)
pub async fn timed_async<F, Fut, R>(name: impl Into<String>, f: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    Timer::measure_async(name, f).await
}

/// Time an async function with threshold (shorthand)
pub async fn timed_async_if_slow<F, Fut, R>(
    name: impl Into<String>,
    threshold_ms: u128,
    f: F,
) -> R
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    Timer::measure_async_if_slow(name, threshold_ms, f).await
}

// ============================================
// AGGREGATE TIMING FOR BATCH OPERATIONS
// ============================================

/// Aggregate timer for tracking multiple operations
pub struct AggregateTimer {
    name: String,
    count: usize,
    total_duration: Duration,
    min_duration: Option<Duration>,
    max_duration: Option<Duration>,
}

impl AggregateTimer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            count: 0,
            total_duration: Duration::ZERO,
            min_duration: None,
            max_duration: None,
        }
    }

    /// Record a single operation duration
    pub fn record(&mut self, duration: Duration) {
        self.count += 1;
        self.total_duration += duration;
        
        self.min_duration = Some(
            self.min_duration
                .map(|min| min.min(duration))
                .unwrap_or(duration)
        );
        
        self.max_duration = Some(
            self.max_duration
                .map(|max| max.max(duration))
                .unwrap_or(duration)
        );
    }

    /// Get average duration
    pub fn avg_duration(&self) -> Option<Duration> {
        if self.count == 0 {
            None
        } else {
            Some(self.total_duration / self.count as u32)
        }
    }

    /// Print summary statistics
    pub fn summary(&self) {
        if self.count == 0 {
            println!("📊 {} - No operations recorded", self.name.cyan());
            return;
        }

        println!("\n{}", "=".repeat(60).blue());
        println!("📊 {} - Summary", self.name.cyan().bold());
        println!("{}", "=".repeat(60).blue());
        println!("  • Count: {}", self.count);
        println!("  • Total: {:.2}s", self.total_duration.as_secs_f64());
        
        if let Some(avg) = self.avg_duration() {
            println!("  • Average: {}ms", avg.as_millis());
        }
        
        if let Some(min) = self.min_duration {
            println!("  • Min: {}ms", min.as_millis());
        }
        
        if let Some(max) = self.max_duration {
            println!("  • Max: {}ms", max.as_millis());
        }
        
        if self.count > 0 {
            let throughput = self.count as f64 / self.total_duration.as_secs_f64();
            println!("  • Throughput: {:.2} ops/sec", throughput);
        }
        
        println!("{}", "=".repeat(60).blue());
    }
}
