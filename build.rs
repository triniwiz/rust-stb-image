/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */



use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub struct Target {
    pub architecture: String,
    pub vendor: String,
    pub system: String,
    pub abi: Option<String>,
}

impl Target {
    pub fn as_strs(&self) -> (&str, &str, &str, Option<&str>) {
        (
            self.architecture.as_str(),
            self.vendor.as_str(),
            self.system.as_str(),
            self.abi.as_deref(),
        )
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}-{}-{}",
            &self.architecture, &self.vendor, &self.system
        )?;

        if let Some(ref abi) = self.abi {
            write!(f, "-{}", abi)
        } else {
            Ok(())
        }
    }
}


fn main() {

    let target_str = std::env::var("TARGET").unwrap();
    let target: Vec<String> = target_str.split('-').map(|s| s.into()).collect();
    if target.len() < 3 {
        assert!(!(target.len() < 3), "Failed to parse TARGET {}", target_str);
    }

    let abi = if target.len() > 3 {
        Some(target[3].clone())
    } else {
        None
    };

    let target = Target {
        architecture: target[0].clone(),
        vendor: target[1].clone(),
        system: target[2].clone(),
        abi,
    };

    println!("cargo:rerun-if-changed=build.rs");

    let mut build = cc::Build::new();

    println!("cargo:rerun-if-changed=src/stb_image.c");

    build
        .cpp(true)
        .define("STB_IMAGE_IMPLEMENTATION", None)
        .file("src/stb_image.c");



    match target.system.as_str() {
        "ios" | "darwin" => {
            let target = std::env::var("TARGET").unwrap();
            let directory = sdk_path(&target).ok();
            add_cc_root(
                directory.as_ref().map(String::as_ref),
                &target,
                &mut build,
            );
        }
        _ => {}
    }

    build.compile("libstb_image");
}


fn sdk_path(target: &str) -> Result<String, std::io::Error> {
    use std::process::Command;
    let sdk = if target.contains("apple-darwin")
        || target == "aarch64-apple-ios-macabi"
        || target == "x86_64-apple-ios-macabi"
    {
        "macosx"
    } else if target == "x86_64-apple-ios"
        || target == "i386-apple-ios"
        || target == "aarch64-apple-ios-sim"
    {
        "iphonesimulator"
    } else if target == "aarch64-apple-ios"
        || target == "armv7-apple-ios"
        || target == "armv7s-apple-ios"
    {
        "iphoneos"
    } else {
        unreachable!();
    };

    let output = Command::new("xcrun")
        .args(&["--sdk", sdk, "--show-sdk-path"])
        .output()?
        .stdout;
    let prefix_str = std::str::from_utf8(&output).expect("invalid output from `xcrun`");
    Ok(prefix_str.trim_end().to_string())
}


fn add_cc_root(sdk_path: Option<&str>, target: &str, builder: &mut cc::Build) {
    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");

    let target = if target == "aarch64-apple-ios" || target == "x86_64-apple-ios" {
        Some(target.to_string())
    } else if target == "aarch64-apple-ios-sim" {
        builder.flag("-m64");
        Some("arm64-apple-ios14.0.0-simulator".to_string())
    } else {
        None
    };

    if let Some(target) = target {
        if target == "x86_64-apple-ios" {
            builder.flag("-mios-simulator-version-min=10.0");
        } else if target == "aarch64-apple-ios" {
            builder.flag("-miphoneos-version-min=10.0");
        }

        builder.flag(&format!("--target={}", target));
    }

    if let Some(sdk_path) = sdk_path {
        builder.flag(&format!("-isysroot{}", sdk_path));
    }
}