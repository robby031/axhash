mod bench_concurrent;
mod bench_hashmap;
mod bench_latency;
mod bench_quality;
mod bench_streaming;
mod bench_throughput;
mod harness;
mod hashers;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║          AxHash Head-to-Head Benchmark Suite                 ║");
    println!("║  Hashers: axhash · xxh3 · wyhash · ahash · fxhash           ║");
    println!("║           siphash-1-3 · highwayhash                          ║");
    println!("║  Timing:  std::time::Instant  (no criterion)                 ║");
    println!("║  Method:  300 samples × 200 batch, median reported           ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    println!("\n[1/7] Throughput...");
    bench_throughput::run();

    println!("\n[2/7] Latency...");
    bench_latency::run();

    println!("\n[3/7] Streaming...");
    bench_streaming::run();

    println!("\n[4/7] Concurrent...");
    bench_concurrent::run();

    println!("\n[5/7] Hash Quality (this may take ~30s)...");
    bench_quality::run();

    println!("\n[6/7] HashMap get-hit...");
    println!("[7/7] HashMap mixed...");
    bench_hashmap::run();

    println!("\n✓ Done.");
}
