use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> anyhow::Result<()> {
	let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	let project_root = manifest_dir.parent().unwrap().parent().unwrap();

	let bindings_dir = project_root.join("ios").join("Bindings");
	let target_dir = project_root.join("target");
	let lib_name = "musicbirb";

	if !bindings_dir.exists() {
		fs::create_dir_all(&bindings_dir)?;
	}

	println!("🦀 Checking/Building host library...");
	run_command(
		Command::new("cargo").args(["build", "-p", "ffi", "--release"]),
		project_root,
	)?;

	let host_lib_ext = if cfg!(target_os = "macos") { "dylib" } else { "so" };
	let host_lib_path = target_dir
		.join("release")
		.join(format!("lib{}.{}", lib_name, host_lib_ext));

	println!("🧬 Generating UniFFI Swift bindings...");
	run_command(
		Command::new("cargo").args([
			"run",
			"-p",
			"ffi",
			"--bin",
			"uniffi-bindgen",
			"generate",
			"--library",
			host_lib_path.to_str().unwrap(),
			"--language",
			"swift",
			"--out-dir",
			bindings_dir.to_str().unwrap(),
		]),
		project_root,
	)?;

	println!("🏗️ Building Rust for iOS targets...");
	let targets = [
		("aarch64-apple-ios", "iphoneos"),
		("aarch64-apple-ios-sim", "iphonesimulator"),
	];

	for (target, sdk_name) in targets {
		println!("  Targeting {} (SDK: {})...", target, sdk_name);

		// Get the absolute path to the SDK using xcrun
		let sdk_path = String::from_utf8(
			Command::new("xcrun")
				.args(["--sdk", sdk_name, "--show-sdk-path"])
				.output()?
				.stdout,
		)?
		.trim()
		.to_string();

		run_command(
			Command::new("cargo")
				.args(["build", "-p", "ffi", "--target", target, "--release"])
				.env("IPHONEOS_DEPLOYMENT_TARGET", "17.0")
				.env("SDKROOT", &sdk_path)
				.env("CFLAGS", format!("-miphoneos-version-min=17.0 -isysroot {}", sdk_path))
				.env(
					"CXXFLAGS",
					format!("-miphoneos-version-min=17.0 -isysroot {}", sdk_path),
				),
			project_root,
		)?;
	}

	let xcframework_path = bindings_dir.join(format!("{}.xcframework", lib_name));
	let mut needs_xcframework = !xcframework_path.exists();

	if !needs_xcframework {
		let fw_mtime = fs::metadata(&xcframework_path)?.modified()?;
		for (target, _) in targets {
			let lib_path = target_dir
				.join(target)
				.join("release")
				.join(format!("lib{}.a", lib_name));
			if fs::metadata(&lib_path)?.modified()? > fw_mtime {
				needs_xcframework = true;
				break;
			}
		}
	}

	if needs_xcframework {
		println!("📦 Creating XCFramework...");
		if xcframework_path.exists() {
			let _ = fs::remove_dir_all(&xcframework_path);
		}

		let headers_dir = bindings_dir.join("headers");
		let _ = fs::remove_dir_all(&headers_dir);
		fs::create_dir_all(&headers_dir)?;

		fs::copy(
			bindings_dir.join(format!("{}FFI.h", lib_name)),
			headers_dir.join(format!("{}FFI.h", lib_name)),
		)?;
		fs::copy(
			bindings_dir.join(format!("{}FFI.modulemap", lib_name)),
			headers_dir.join("module.modulemap"),
		)?;

		run_command(
			Command::new("xcodebuild").args([
				"-create-xcframework",
				"-library",
				target_dir
					.join("aarch64-apple-ios")
					.join("release")
					.join(format!("lib{}.a", lib_name))
					.to_str()
					.unwrap(),
				"-headers",
				headers_dir.to_str().unwrap(),
				"-library",
				target_dir
					.join("aarch64-apple-ios-sim")
					.join("release")
					.join(format!("lib{}.a", lib_name))
					.to_str()
					.unwrap(),
				"-headers",
				headers_dir.to_str().unwrap(),
				"-output",
				xcframework_path.to_str().unwrap(),
			]),
			project_root,
		)?;
		let _ = fs::remove_dir_all(headers_dir);
	} else {
		println!("⏭️ XCFramework is up to date.");
	}

	Ok(())
}

fn run_command(cmd: &mut Command, current_dir: &Path) -> anyhow::Result<()> {
	let status = cmd.current_dir(current_dir).status()?;
	if !status.success() {
		anyhow::bail!("Command failed: {:?}", cmd);
	}
	Ok(())
}
