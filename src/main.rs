// gini: A simple CLI checkpoint system for your projects.
// Author: Somendra somendra830@gmail.com
// Version: 0.1.4
//
// This tool lets you create, list, and restore checkpoints in your project directory.
// Each checkpoint is a folder under .gini/checkpoints with a timestamp and name.
// Optionally, it can stash your git state when creating a checkpoint.

use clap::{Arg, ArgAction, Command};
use fs_extra::dir::CopyOptions;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ShellCommand;

const CHECKPOINT_DIR: &str = ".gini/checkpoints";

/// The main entry point of the `gini` CLI application.
///
/// It parses command-line arguments and executes the corresponding subcommand:
/// `init`, `checkpoint`, `restore`, or `list`.
fn main() {
    let matches = Command::new("gini")
        .version("0.1.4")
        .author("Somendra somendra830@gmail.com")
        .about("A simple CLI checkpoint system")
        .arg_required_else_help(true)
        .subcommand(Command::new("init").about("Initialize a gini project"))
        .arg(
            Arg::new("checkpoint")
                .short('c')
                .long("checkpoint")
                .value_name("NAME")
                .help("Create a checkpoint")
                .conflicts_with_all(["restore", "list"])
                .num_args(1..),
        )
        .arg(
            Arg::new("restore")
                .short('r')
                .long("restore")
                .value_name("NAME")
                .help("Restore a checkpoint")
                .conflicts_with_all(["checkpoint", "list"])
                .num_args(1..),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .action(ArgAction::SetTrue)
                .help("List all checkpoints")
                .conflicts_with_all(["checkpoint", "restore"]),
        )
        .get_matches();

    if let Some(("init", _)) = matches.subcommand() {
        init_project();
    } else if let Some(values) = matches.get_many::<String>("checkpoint") {
        let name = values.map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
        ensure_initialized();
        create_checkpoint(&name);
    } else if let Some(values) = matches.get_many::<String>("restore") {
        let name = values.map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
        ensure_initialized();
        restore_checkpoint(&name);
    } else if matches.get_flag("list") {
        ensure_initialized();
        list_checkpoints();
    }
}

/// Initializes the project by creating the `.gini/checkpoints` directory.
///
/// If the directory already exists, it prints a message and does nothing.
fn init_project() {
    let path = Path::new(CHECKPOINT_DIR);
    if path.exists() {
        println!("--- .gini already exists.");
    } else {
        fs::create_dir_all(path).expect("Failed to create .gini folder");
        println!(
            "--- Initialized empty .gini project in {}",
            std::env::current_dir().unwrap().display()
        );
    }
}

/// Ensures that the `.gini/checkpoints` directory exists.
///
/// If the directory is not found, it prints an error message and exits the process.
fn ensure_initialized() {
    if !Path::new(CHECKPOINT_DIR).exists() {
        eprintln!("--- No .gini project found in this directory.\n--- Run `gini init` first.");
        std::process::exit(1);
    }
}

/// Creates a new checkpoint.
///
/// This involves:
/// 1. Creating a timestamped folder for the checkpoint.
/// 2. Copying all project files (except `.gini` and `.git`) into it.
/// 3. If in a git repository, stashing the current changes with a checkpoint message.
///
/// # Arguments
///
/// * `name` - The name for the new checkpoint.
fn create_checkpoint(name: &str) {
    let sanitized_name = name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let folder_name = format!("{}_{}", timestamp, sanitized_name);
    let checkpoint_path = Path::new(CHECKPOINT_DIR).join(&folder_name);
    fs::create_dir_all(&checkpoint_path).expect("Failed to create checkpoint folder");

    println!("Creating snapshot...");
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();

    for entry in fs::read_dir(".").unwrap().flatten() {
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name != ".gini" && file_name != ".git" {
                paths_to_copy.push(path);
            }
        }
    }

    if !paths_to_copy.is_empty() {
        fs_extra::copy_items(&paths_to_copy, &checkpoint_path, &copy_options)
            .expect("Failed to copy files to checkpoint");
    }

    if Path::new(".git").exists() {
        let _ = ShellCommand::new("git")
            .arg("stash")
            .arg("push")
            .arg(format!("--message=gini: checkpoint '{}'", name))
            .output()
            .expect("Failed to stash git state");
    }

    println!(
        "--- Checkpoint \"{}\" saved at {}",
        name,
        checkpoint_path.display()
    );
}

/// Restores the project state from a specified checkpoint.
///
/// It finds the checkpoint by name, then copies all its contents back to the
/// project's root directory. If a git stash was created, it attempts to pop it.
///
/// # Arguments
///
/// * `name` - The name of the checkpoint to restore.
fn restore_checkpoint(name: &str) {
    let checkpoint = find_checkpoint_by_name(name);
    match checkpoint {
        Some(path) => {
            println!("Restoring snapshot from {}...", path.display());
            let mut copy_options = CopyOptions::new();
            copy_options.overwrite = true;
            copy_options.copy_inside = true;

            let mut paths_to_copy = Vec::new();
            for entry in fs::read_dir(&path).unwrap().flatten() {
                paths_to_copy.push(entry.path());
            }

            if !paths_to_copy.is_empty() {
                let current_dir = std::env::current_dir().unwrap();
                fs_extra::copy_items(&paths_to_copy, current_dir, &copy_options)
                    .expect("Failed to copy files from checkpoint");
            }

            if Path::new(".git").exists() {
                let _ = ShellCommand::new("git")
                    .arg("stash")
                    .arg("pop")
                    .output()
                    .expect("Failed to restore git stash");
            }
            println!("--- Restored checkpoint \"{}\" from {}. Please verify manually.", name, path.display());
        }
        None => {
            eprintln!("--- Checkpoint \"{}\" not found.", name);
            std::process::exit(1);
        }
    }
}

/// Lists all available checkpoints in the `.gini/checkpoints` directory.
///
/// It prints each checkpoint's folder name to the console.
fn list_checkpoints() {
    let path = Path::new(CHECKPOINT_DIR);
    if let Ok(entries) = fs::read_dir(path) {
        println!("--- Available checkpoints:");
        let mut found = false;
        for entry in entries.flatten() {
            println!("- {}", entry.file_name().to_string_lossy());
            found = true;
        }
        if !found {
            println!("(none)");
        }
    } else {
        println!("--- No checkpoints found.");
    }
}

/// Finds the full path of a checkpoint by its name.
///
/// It searches for a directory in the checkpoint folder that either matches the name
/// exactly or ends with `_{name}`.
///
/// # Arguments
///
/// * `name` - The name of the checkpoint to find.
///
/// # Returns
///
/// An `Option<PathBuf>` which is `Some(path)` if found, or `None` otherwise.
fn find_checkpoint_by_name(name: &str) -> Option<PathBuf> {
    let sanitized_name = name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
    let path = Path::new(CHECKPOINT_DIR);
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Some(fname_str) = entry.path().file_name().and_then(|f| f.to_str()) {
                // Exact match (for full checkpoint name)
                if fname_str == name {
                    return Some(entry.path());
                }
                // Match against sanitized name
                if fname_str == sanitized_name {
                    return Some(entry.path());
                }
                // Suffix match for partial name
                if fname_str.ends_with(&format!("_{}", name)) {
                    return Some(entry.path());
                }
                // Suffix match for sanitized partial name
                if fname_str.ends_with(&format!("_{}", sanitized_name)) {
                    return Some(entry.path());
                }
            }
        }
    }
    None
}
