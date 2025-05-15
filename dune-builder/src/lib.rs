// Copyright (c) Viable Systems and TezEdge Contributors
// SPDX-License-Identifier: MIT

use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// Specifies how the `dune` command should be invoked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DuneInvocation {
    /// Invoke `dune` directly from the system path.
    System,
    /// Invoke `dune` via `opam exec -- dune`. This is the default.
    OpamExec,
}

fn find_dune_project_location<P: AsRef<Path>>(base_dir: P) -> Option<String> {
    let mut path = base_dir.as_ref().to_path_buf();
    while let Some(parent) = path.parent() {
        if parent.join("dune-project").exists() {
            return Some(parent.to_string_lossy().into_owned());
        }
        path = parent.to_path_buf();
    }
    None
}

fn dir_relative_to_dune_project<P: AsRef<Path>, Q: AsRef<Path>>(
    base_dir: P,
    dune_project_dir: Q,
) -> Option<String> {
    let path = base_dir.as_ref();
    if let Ok(relative_path) = path.strip_prefix(dune_project_dir.as_ref()) {
        return Some(relative_path.to_string_lossy().into_owned());
    }
    None
}

/// Helper for building OCaml code with dune and collecting object files for linking with Rust.
pub struct DuneBuilder {
    /// Absolute path to the directory containing the dune-project file.
    dune_project_dir: PathBuf,
    /// Absolute path to the dune build output directory (e.g. <dune_project_dir>/_build/<profile>).
    dune_build_dir: PathBuf,
    /// Path to the OCaml source directory, relative to the dune project root (e.g. "ocaml").
    ocaml_build_dir: PathBuf,
    /// Dune build profile (e.g., "default", "release").
    profile: String,
    /// Custom arguments to pass to `dune build`.
    dune_args: Vec<String>,
    /// How to invoke the `dune` command.
    dune_invocation: DuneInvocation,
}

impl DuneBuilder {
    /// Creates a new DuneBuilder with the "default" profile and no custom dune arguments.
    ///
    /// `ocaml_dir` is the path to the OCaml source directory relative to the crate's manifest directory.
    /// By default, `dune` is invoked via `opam exec -- dune`.
    pub fn new<P: AsRef<Path>>(ocaml_dir: P) -> Self {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
            panic!("CARGO_MANIFEST_DIR not set. This crate is intended for use in build scripts.");
        });
        let callable_ocaml_dir = Path::new(&manifest_dir).join(ocaml_dir.as_ref());

        let dune_project_dir_str = find_dune_project_location(&callable_ocaml_dir)
            .expect("Failed to find dune project location");
        let dune_project_dir = PathBuf::from(dune_project_dir_str);

        let ocaml_build_dir_str = dir_relative_to_dune_project(
            &callable_ocaml_dir,
            &dune_project_dir,
        )
        .expect("Failed to determine OCaml build directory relative to dune project");
        let ocaml_build_dir = PathBuf::from(ocaml_build_dir_str);

        let profile = "default".to_string();
        let dune_build_dir = dune_project_dir.join("_build").join(&profile);

        Self {
            dune_project_dir,
            dune_build_dir,
            ocaml_build_dir,
            profile,
            dune_args: Vec::new(),
            dune_invocation: DuneInvocation::OpamExec, // Default invocation
        }
    }

    /// Returns a new `DuneBuilder` instance with the specified build profile.
    ///
    /// The build output directory will be adjusted to `<dune_project_dir>/_build/<profile_name>`.
    pub fn with_profile(self, profile_name: &str) -> Self {
        let new_profile = profile_name.to_string();
        let new_dune_build_dir = self.dune_project_dir.join("_build").join(&new_profile);
        Self {
            dune_build_dir: new_dune_build_dir,
            profile: new_profile,
            ..self
        }
    }

    /// Returns a new `DuneBuilder` instance with the specified custom arguments for `dune build`.
    ///
    /// These arguments will be appended to the `dune build` command.
    pub fn with_dune_args(self, extra_args: Vec<String>) -> Self {
        Self {
            dune_args: extra_args,
            ..self
        }
    }

    /// Returns a new `DuneBuilder` instance with the specified Dune invocation method.
    pub fn with_dune_invocation(self, invocation: DuneInvocation) -> Self {
        Self {
            dune_invocation: invocation,
            ..self
        }
    }

    /// Builds the specified dune target and returns a list of resulting object files.
    pub fn build<T: AsRef<Path>>(&self, target: T) -> Vec<PathBuf> {
        let target_ref = target.as_ref();
        // target_path_buf is relative to ocaml_build_dir, which is relative to dune_project_dir.
        // This path is what `dune build` expects for the target.
        let target_path_for_dune = self.ocaml_build_dir.join(target_ref);
        let target_path_str = target_path_for_dune.to_str().unwrap_or_else(|| {
            panic!(
                "Constructed target path for dune is not valid UTF-8: {}",
                target_path_for_dune.display()
            )
        });

        let mut command = match self.dune_invocation {
            DuneInvocation::System => Command::new("dune"),
            DuneInvocation::OpamExec => {
                let mut cmd = Command::new("opam");
                cmd.arg("exec").arg("--");
                cmd.arg("dune");
                cmd
            }
        };

        command.arg("build");

        if self.profile != "default" {
            command.arg("--profile").arg(&self.profile);
        }
        
        command.arg(target_path_str.to_string());
        command.args(&self.dune_args);

        let status = command
            .current_dir(&self.dune_project_dir)
            .status()
            .expect("Dune build command failed to start");

        if !status.success() {
            panic!(
                "Dune build failed for target: {}. Profile: {}. Args: {:?}. Exit status: {:?}",
                target_path_str,
                self.profile,
                self.dune_args,
                status.code()
            );
        }

        // Find all .o files in the build output directory for the ocaml dir.
        // self.dune_build_dir is already profile-aware.
        let build_output_dir = self.dune_build_dir.join(&self.ocaml_build_dir);
        let mut objects = Vec::new();
        match std::fs::read_dir(&build_output_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "o" {
                                objects.push(path);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                panic!(
                    "Failed to read dune build output directory '{}': {}",
                    build_output_dir.display(),
                    e
                );
            }
        }

        if objects.is_empty() {
            eprintln!(
                "Warning: No .o files found in {}. Ensure the dune target '{}' produces .o files in this directory.",
                build_output_dir.display(),
                target_ref.display()
            );
        }
        objects
    }
}
