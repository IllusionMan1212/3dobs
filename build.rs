use std::{env, process::Command};

fn main() {
    // Get the current directory of the project
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .current_dir(&project_dir)
        .output()
        .expect("Failed to execute git command");

    if output.status.success() {
        let commit_hash = String::from_utf8(output.stdout).expect("Git output is not valid UTF-8");

        // Export the commit hash as a build-time constant
        println!("cargo:rustc-env=GIT_HASH={}", commit_hash);
    } else {
        // Handle the case where the git command failed
        eprintln!("Failed to get Git commit hash");
    }
}
