fn main() {
    tauri_build::build();
    
    // Only run this for iOS builds
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "ios" {
        // Tell cargo to link the required frameworks
        println!("cargo:rustc-link-arg=-framework");
        println!("cargo:rustc-link-arg=PhotosUI");
        println!("cargo:rustc-link-arg=-framework");
        println!("cargo:rustc-link-arg=Photos");
        println!("cargo:rustc-link-arg=-framework");
        println!("cargo:rustc-link-arg=UIKit");
        println!("cargo:rustc-link-arg=-framework");
        println!("cargo:rustc-link-arg=MobileCoreServices");
        
        // Compile Swift file directly to static library
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let swift_file = std::path::Path::new(&manifest_dir).join("swift-packages/ImagePicker/Sources/ImagePicker/ImagePicker.swift");
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let lib_path = std::path::Path::new(&out_dir).join("libImagePicker.a");
        
        // Only recompile if Swift file changed
        println!("cargo:rerun-if-changed={}", swift_file.display());
        
        // Compile Swift to object file
        let sdk_path_output = std::process::Command::new("xcrun")
            .args(["--sdk", "iphonesimulator", "--show-sdk-path"])
            .output()
            .expect("Failed to get SDK path");
        let sdk_path = String::from_utf8_lossy(&sdk_path_output.stdout).trim().to_string();
        
        let target = "arm64-apple-ios15.0-simulator";
        let config = if std::env::var("DEBUG").unwrap_or_default() == "true" { "debug" } else { "release" };
        
        // Compile Swift file to object file
        let compile_status = std::process::Command::new("xcrun")
            .args([
                "swiftc",
                "-target", target,
                "-sdk", &sdk_path,
                "-c", swift_file.to_str().unwrap(),
                "-o", lib_path.with_extension("o").to_str().unwrap(),
                "-parse-as-library",
                "-module-name", "ImagePicker",
                "-emit-module",
                "-emit-module-path", lib_path.with_extension("swiftmodule").to_str().unwrap(),
            ])
            .status()
            .expect("Failed to compile Swift file");
        
        if !compile_status.success() {
            panic!("Failed to compile Swift file");
        }
        
        // Create static library from object file
        let lib_status = std::process::Command::new("xcrun")
            .args([
                "ar", "rcs",
                lib_path.to_str().unwrap(),
                lib_path.with_extension("o").to_str().unwrap(),
            ])
            .status()
            .expect("Failed to create static library");
        
        if !lib_status.success() {
            panic!("Failed to create static library");
        }
        
        // Tell cargo to link the static library
        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=static=ImagePicker");
    }
}