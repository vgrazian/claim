use std::process::Command;

fn main() {
    // Get the build timestamp
    let output = Command::new("date")
        .arg("+%Y-%m-%d %H:%M:%S %Z")
        .output()
        .expect("Failed to execute date command");
    
    let build_date = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8")
        .trim()
        .to_string();
    
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
    
    // Re-run if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}

// Made with Bob
