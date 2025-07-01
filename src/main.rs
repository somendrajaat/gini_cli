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
// use std::process::Command as ShellCommand;
use dialoguer::Select;

const CHECKPOINT_DIR: &str = ".gini/checkpoints";

/// The main entry point of the `gini` CLI application.
///
/// It parses command-line arguments and executes the corresponding subcommand:
/// `init`, `checkpoint`, `restore`, or `list`.
fn main() {
    let matches = Command::new("gini")
        .version("0.1.6")
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
                .help("Restore a checkpoint")
                .conflicts_with_all(["checkpoint", "list"])
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .action(ArgAction::SetTrue)
                .help("List all checkpoints")
                .conflicts_with_all(["checkpoint", "restore"]),
        )
        .arg(
            Arg::new("delete")
                .short('d')
                .long("delete")
                .help("Delete a checkpoint")
                .conflicts_with_all(["checkpoint", "restore", "list"])
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    if let Some(("init", _)) = matches.subcommand() {
        init_project();
    } else if let Some(values) = matches.get_many::<String>("checkpoint") {
        let name = values.map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
        ensure_initialized();
        create_checkpoint(&name);
    } else if matches.contains_id("restore") {
        ensure_initialized();
        restore_checkpoint_tui();
    } else if matches.get_flag("list") {
        ensure_initialized();
        list_checkpoints();
    } else if matches.contains_id("delete") {
        ensure_initialized();
        delete_checkpoint_tui();
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
            "gini: Initialized empty .gini project in {}",
            std::env::current_dir().unwrap().display()
        );
    }
}

/// Ensures that the `.gini/checkpoints` directory exists.
///
/// If the directory is not found, it prints an error message and exits the process.
fn ensure_initialized() {
    if !Path::new(CHECKPOINT_DIR).exists() {
        eprintln!("gini: No .gini project found in this directory.\n--- Run `gini init` first.");
        std::process::exit(1);
    }
}

/// Creates a new checkpoint.
///
/// This involves:
/// 1. Creating a timestamped folder for the checkpoint.
/// 2. Copying all project files (except `.gini` and `.git`) into it.
///
/// # Arguments
///
/// * `name` - The name for the new checkpoint.
fn create_checkpoint(name: &str) {
    let sanitized_name = name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let folder_name = format!("{}_{}", timestamp, sanitized_name);
    let checkpoint_path = Path::new(CHECKPOINT_DIR).join(&folder_name);
    fs::create_dir_all(&checkpoint_path).expect("gini: Failed to create checkpoint folder");

    println!("gini: Creating snapshot with regular copies...");
    let entries = match fs::read_dir(".") {
        Ok(e) => e,
        Err(e) => {
            eprintln!("gini: Failed to read current directory: {}", e);
            std::process::exit(1);
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name != ".gini" && file_name != ".git" {
                let dst = checkpoint_path.join(file_name);
                copy_recursively(&path, &dst).expect("gini: Failed to copy files");
            }
        }
    }

    println!(
        "gini: Checkpoint \"{}\" saved at {}",
        name,
        checkpoint_path.display()
    );
}

/// Recursively copy src to dst using regular file copies (no hard links).
fn copy_recursively(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if file_type.is_dir() {
                copy_recursively(&src_path, &dst_path)?;
            } else if file_type.is_file() {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    } else if src.is_file() {
        fs::copy(src, dst)?;
    }
    Ok(())
}

/// Restores the project state from a specified checkpoint.
///
/// It finds the checkpoint by name, then copies all its contents back to the
/// project's root directory.
///
/// # Arguments
///
/// * `name` - The name of the checkpoint to restore.
fn restore_checkpoint(name: &str) {
    let checkpoint = find_checkpoint_by_name(name);
    match checkpoint {
        Some(path) => {
            println!("gini: Restoring snapshot from {}...", path.display());
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
                    .expect("gini: Failed to copy files from checkpoint");
            }
            println!("gini: Restored checkpoint \"{}\" from {}. Please verify manually.", name, path.display());
        }
        None => {
            eprintln!("gini: Checkpoint \"{}\" not found.", name);
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
        println!("gini: Available checkpoints:");
        let mut found = false;
        for entry in entries.flatten() {
            println!("- {}", entry.file_name().to_string_lossy());
            found = true;
        }
        if !found {
            println!("(none)");
        }
    } else {
        println!("gini: No checkpoints found.");
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

/// Helper to list checkpoint folder names as Vec<String>
fn list_checkpoint_names() -> Vec<String> {
    let path = Path::new(CHECKPOINT_DIR);
    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    names
}

/// Restores the project state from a selected checkpoint using a TUI.
fn restore_checkpoint_tui() {
    let checkpoints = list_checkpoint_names();
    if checkpoints.is_empty() {
        println!("gini: No checkpoints found.");
        return;
    }
    let selection = match Select::new()
        .with_prompt("gini: Select a checkpoint to restore")
        .items(&checkpoints)
        .default(0)
        .interact()
    {
        Ok(idx) => idx,
        Err(_) => {
            println!("gini: Restore cancelled.");
            return;
        }
    };
    let name = &checkpoints[selection];
    restore_checkpoint(name);
}

/// Deletes a checkpoint selected via TUI.
fn delete_checkpoint_tui() {
    let checkpoints = list_checkpoint_names();
    if checkpoints.is_empty() {
        println!("gini: No checkpoints found.");
        return;
    }
    let selection = match Select::new()
        .with_prompt("gini: Select a checkpoint to delete")
        .items(&checkpoints)
        .default(0)
        .interact()
    {
        Ok(idx) => idx,
        Err(_) => {
            println!("gini: Delete cancelled.");
            return;
        }
    };
    let name = &checkpoints[selection];
    let checkpoint_path = Path::new(CHECKPOINT_DIR).join(name);
    match fs::remove_dir_all(&checkpoint_path) {
        Ok(_) => println!("gini: Checkpoint '{}' deleted.", name),
        Err(e) => println!("gini: Failed to delete checkpoint '{}': {}", name, e),
      
    }
}
