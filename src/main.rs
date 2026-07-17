mod homework_3;
mod homework_4;
mod pcg;
mod rng;

use std::io::Write;
use std::process::Child;

fn main() {
    println!("running homework 3...");
    homework_3::run();

    println!();
    println!();
    println!();

    println!("running homework 4...");
    homework_4::run();

    println!("done!");
}

fn exit_gnuplot(mut gnuplot: Child) {
    let stdin = gnuplot.stdin.as_mut().expect("stdin to exist");
    writeln!(stdin, "exit").unwrap();
    let timeout = std::time::Duration::from_secs(5);
    let start = std::time::Instant::now();
    loop {
        if let Some(exit_status) = gnuplot.try_wait().unwrap() {
            println!("gnuplot exited with exit status: {}", exit_status);
            break;
        }

        let now = std::time::Instant::now();
        let diff = now - start;
        if diff > timeout {
            println!("failed to wait for gnuplot to exit in {:?}", timeout);
            break;
        }

        std::thread::yield_now();
    }
}
