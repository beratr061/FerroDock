#![allow(clippy::print_stdout, clippy::print_stderr)]

mod release;
mod rules;
mod scaffold;
mod templates;

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "usage: cargo xtask <check-rules | new-feature <name> | release-key <output> | release-public-key <key-file> | sign-manifest <version> <digest> <output>>";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
}

fn report(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = workspace_root();
    match args.as_slice() {
        [command] if command == "check-rules" => report(rules::check(&root)),
        [command, name] if command == "new-feature" => report(scaffold::create(&root, name)),
        [command, output] if command == "release-key" => {
            report(release::generate_key(&PathBuf::from(output)))
        }
        [command, key_file] if command == "release-public-key" => {
            report(release::print_public_key(&PathBuf::from(key_file)))
        }
        [command, version, digest, output] if command == "sign-manifest" => report(
            release::sign_manifest(&root, version, digest, &PathBuf::from(output)),
        ),
        _ => report(Err(USAGE.to_owned())),
    }
}
