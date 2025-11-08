use std::process::{exit, Command};

fn main() {
    println!("🧪 --- Running standard tests ---");
    let status = Command::new("cargo")
        .arg("test")
        .status()
        .expect("Failed to execute 'cargo test'");

    if !status.success() {
        eprintln!("❌ 'cargo test' failed.");
        exit(1);
    }

    println!("\n🕸️  --- Running web tests ---");
    let status_web = Command::new("cargo")
        .arg("test-web")
        .status()
        .expect("Failed to execute 'cargo test-web'");

    if !status_web.success() {
        eprintln!("❌ 'cargo test-web' failed.");
        exit(1);
    }

    println!("\n🎉 All tests passed! 🎉");
}
