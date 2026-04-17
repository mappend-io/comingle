use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=viewer/src");
    println!("cargo:rerun-if-changed=viewer/index.html");
    println!("cargo:rerun-if-changed=viewer/vite.config.js");
    println!("cargo:rerun-if-changed=viewer/package.json");

    let install_status = Command::new("bun")
        .args(["install"])
        .args(["--frozen-lockfile"])
        .current_dir("viewer")
        .status()
        .expect("failed to spawn 'bun install', is bun installed?");

    assert!(install_status.success(), "viewer install failed");

    let build_status = Command::new("bun")
        .args(["run", "build"])
        .current_dir("viewer")
        .status()
        .expect("failed to spawn 'bun run build', is bun installed?");

    assert!(build_status.success(), "viewer build failed");
}
