use std::process::{exit, Command};

fn main() {
    let mut cmd = Command::new("wasm-pack");
    cmd.args([
        "test",
        "--chrome",
        "--headless",
        "--",
        "--no-default-features",
        "--features",
        "web",
    ]);

    println!("Running: {cmd:?}");

    let status = cmd.status().expect("Failed to execute wasm-pack command");

    if !status.success() {
        eprintln!("Command failed with status: {status}");
        exit(1);
    }
}
