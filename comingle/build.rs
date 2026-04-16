use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=viewer/src");
    println!("cargo:rerun-if-changed=viewer/index.html");
    println!("cargo:rerun-if-changed=viewer/vite.config.js");
    println!("cargo:rerun-if-changed=viewer/package.json");

    let install_status = Command::new("npm")
        .args(["install"])
        .current_dir("viewer")
        .status()
        .expect("failed to run npm install, is Node installed?");

    assert!(install_status.success(), "viewer install failed");

    let build_status = Command::new("npm")
        .args(["run", "build"])
        .current_dir("viewer")
        .status()
        .expect("failed to run npm build, is Node installed?");

    assert!(build_status.success(), "viewer build failed");
}
