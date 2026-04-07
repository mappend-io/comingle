use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=viewer/src");
    println!("cargo:rerun-if-changed=viewer/index.html");
    println!("cargo:rerun-if-changed=viewer/vite.config.js");
    println!("cargo:rerun-if-changed=viewer/package.json");

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir("viewer")
        .status()
        .expect("failed to run npm build, is Node installed?");

    assert!(status.success(), "viewer build failed");
}
