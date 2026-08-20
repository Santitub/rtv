// Medidor: ¿a qué ritmo consume samples el callback de cpal?
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() {
    let host = cpal::default_host();
    let dev = host.default_output_device().expect("no device");
    let cfg = dev.default_output_config().expect("no config");
    let rate = cfg.sample_rate().0;
    let ch = cfg.channels();
    println!("rate={} ch={}", rate, ch);
    let sc = cpal::StreamConfig { channels: ch, sample_rate: cfg.sample_rate(), buffer_size: cpal::BufferSize::Default };
    let count = Arc::new(AtomicU64::new(0));
    let c2 = count.clone();
    let stream = dev.build_output_stream(&sc, move |out: &mut [f32], info: &cpal::OutputCallbackInfo| {
        out.fill(0.0);
        c2.fetch_add(out.len() as u64, Ordering::Relaxed);
        let ts = info.timestamp();
        let d = ts.playback.duration_since(&ts.callback).map(|d| d.as_secs_f64()).unwrap_or(-1.0);
        static PRINTED: AtomicU64 = AtomicU64::new(0);
        if PRINTED.fetch_add(1, Ordering::Relaxed).is_multiple_of(50) {
            eprintln!("cb: buf={} delay={:.4}s", out.len(), d);
        }
    }, |e| eprintln!("err {e}"), None).expect("build");
    stream.play().unwrap();
    let mut last = 0u64;
    for i in 0..12 {
        std::thread::sleep(Duration::from_secs(1));
        let n = count.load(Ordering::Relaxed) / ch as u64;
        println!("t={}s interval={} frames/s (expected {})", i + 1, n - last, rate);
        last = n;
    }
}
