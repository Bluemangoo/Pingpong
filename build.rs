use std::process::Command;

fn main() {
    if let Ok(git_ref) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output() {
        println!(
            "cargo:rustc-env=GIT_REF={}",
            String::from_utf8_lossy(&git_ref.stdout)
        );
    }
}
