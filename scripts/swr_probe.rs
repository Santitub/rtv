// Probe: reproduce the audio decode+resample path and measure
// output samples vs media time.
use ffmpeg_the_third as ffmpeg;
use ffmpeg::format::sample::Type as SampleType;
use ffmpeg::format::{input, Sample as SampleFormat};
use ffmpeg::media::Type as MediaType;
use ffmpeg::software::resampling::context::Context as SwrCtx;
use ffmpeg::util::frame::audio::Audio as AudioFrame;
use ffmpeg::ChannelLayout;

fn main() {
    ffmpeg::init().unwrap();
    let path = std::env::args().nth(1).unwrap();
    let out_rate: u32 = std::env::args().nth(2).map(|s| s.parse().unwrap()).unwrap_or(44100);
    let mut ictx = input(&path).unwrap();
    let stream = ictx.streams().best(MediaType::Audio).unwrap();
    let idx = stream.index();
    let tb = stream.time_base();
    let dec_ctx = ffmpeg::codec::context::Context::from_parameters(stream.parameters()).unwrap();
    let mut decoder = dec_ctx.decoder().audio().unwrap();
    let in_rate = decoder.rate();
    println!("in_rate={} fmt={:?} out_rate={}", in_rate, decoder.format(), out_rate);
    let mut swr = SwrCtx::get2(
        decoder.format(), decoder.ch_layout().to_owned(), in_rate,
        SampleFormat::F32(SampleType::Packed), ChannelLayout::STEREO, out_rate,
    ).unwrap();
    let mut in_frame = AudioFrame::empty();
    let mut out_frame = AudioFrame::empty();
    let mut total_in: u64 = 0;
    let mut total_out: u64 = 0;
    let mut nframes = 0;
    for r in ictx.packets() {
        let (s, p) = r.unwrap();
        if s.index() != idx { continue; }
        let _ = decoder.send_packet(&p);
        while decoder.receive_frame(&mut in_frame).is_ok() {
            let delay = swr.run(&in_frame, &mut out_frame).unwrap();
            total_in += in_frame.samples() as u64;
            total_out += out_frame.samples() as u64;
            nframes += 1;
            if nframes <= 8 || nframes % 200 == 0 {
                println!("f#{} in={} out={} delay={:?} total_in={} total_out={} expected_out={}",
                    nframes, in_frame.samples(), out_frame.samples(), delay.map(|d| d.output),
                    total_in, total_out,
                    total_in * out_rate as u64 / in_rate as u64);
            }
            if nframes >= 1000 { let _ = tb; return; }
        }
    }
}
