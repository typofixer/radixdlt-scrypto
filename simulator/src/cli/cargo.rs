use std::path::PathBuf;
use std::process::Command;

use crate::cli::*;

pub fn build_cargo_package(mut path: PathBuf) -> Result<PathBuf, BuildPackageError> {
    let mut cargo = path.clone();
    cargo.push("Cargo.toml");

    if cargo.exists() {
        Command::new("cargo")
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("--release")
            .arg("--manifest-path")
            .arg(cargo.canonicalize().unwrap().to_str().unwrap())
            .spawn()
            .map_err(|e| BuildPackageError::FailedToRunCargo(e))?
            .wait()
            .map_err(|e| BuildPackageError::FailedToWaitCargo(e))?;

        let manifest = cargo_toml::Manifest::from_path(cargo)
            .map_err(|e| BuildPackageError::FailedToParseCargoToml(e))?;
        path.push("target");
        path.push("wasm32-unknown-unknown");
        path.push("release");
        path.push(
            manifest
                .package
                .ok_or(BuildPackageError::MissingPackageInCargoToml)?
                .name,
        );
        Ok(path.with_extension("wasm"))
    } else {
        Err(BuildPackageError::NotCargoPackage)
    }
}