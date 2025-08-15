/*
=========================================================================
File: Cargo.toml (Root of your project)
Description: This is the only Cargo.toml you need.
-------------------------------------------------------------------------
Key changes:
- All dependencies are now in this single file.
- The `gini_lib` dependency has been removed as the code is merged.
=========================================================================
*/

/*
[package]
name = "gini"
version = "0.2.0"
edition = "2021"
author = "Somendra somendra830@gmail.com"
about = "A simple CLI checkpoint system"

[dependencies]
clap = { version = "4.4.8", features = ["derive"] }
anyhow = "1.0.75"
dialoguer = "0.11.0"
sha1 = "0.10.6"
hex = "0.4.3"
flate2 = "1.0.28"
chrono = "0.4"
*/


/*
=========================================================================
File: src/main.rs
Description: Your complete, single-file application.
-------------------------------------------------------------------------
All code from the `gini_lib` has been moved into this file.
This resolves all compilation errors related to unresolved imports and modules.
=========================================================================
*/

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::fs;
use std::io::stdin;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono;

// --- Constants and Configuration ---

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB limit
const MAX_COMMIT_MESSAGE_LENGTH: usize = 1000;
const HASH_LENGTH: usize = 40;

// --- CLI Definition ---

/// A simple, efficient CLI checkpoint system for your projects.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new Gini repository.
    Init,
    /// Create a new checkpoint with a message.
    #[command(alias = "c")]
    Checkpoint {
        #[arg(short, long)]
        message: String,
    },
    /// Restore the project to a previous checkpoint.
    #[command(alias = "r")]
    Restore,
    /// List all checkpoints in the project's history.
    #[command(alias = "l")]
    Log,
    /// Restore from a backup.
    #[command(alias = "b")]
    Backup,
}

// --- Main Application Logic ---

fn main() -> Result<()> {
    // Set up proper error handling
    if let Err(e) = run() {
        eprintln!("gini: error: {}", e);
        std::process::exit(1);
    }
    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    
    // Validate input
    if let Commands::Checkpoint { ref message } = cli.command {
        if message.is_empty() {
            bail!("Commit message cannot be empty");
        }
        if message.len() > MAX_COMMIT_MESSAGE_LENGTH {
            bail!("Commit message too long (max {} characters)", MAX_COMMIT_MESSAGE_LENGTH);
        }
    }

    if !matches!(cli.command, Commands::Init) {
        ensure_initialized()?;
    }

    match cli.command {
        Commands::Init => {
            init()?;
        }
        Commands::Checkpoint { message } => {
            let commit_hash = checkpoint(&message)?;
            println!("gini: Checkpoint created with hash: {}", commit_hash);
        }
        Commands::Restore => {
            restore_checkpoint_tui()?;
        }
        Commands::Log => {
            let log_output = log()?;
            println!("{}", log_output);
        }
        Commands::Backup => {
            restore_backup_tui()?;
        }
    }

    Ok(())
}

/// Restores the project state from a selected checkpoint using a TUI.
fn restore_checkpoint_tui() -> Result<()> {
    let commits = get_commit_history()?;
    
    if commits.is_empty() {
        println!("gini: No checkpoints found to restore.");
        return Ok(());
    }

    // Display available checkpoints
    println!("gini: Available checkpoints:");
    for (i, (hash, msg)) in commits.iter().enumerate() {
        println!("  {}. {} - {}", i + 1, &hash[..7], msg);
    }

    // Simple text-based selection
    println!("\ngini: Enter checkpoint number to restore (1-{}):", commits.len());
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    let selection: usize = input.trim().parse()
        .map_err(|_| anyhow::anyhow!("Invalid selection"))?;
    
    if selection < 1 || selection > commits.len() {
        bail!("Invalid selection: must be between 1 and {}", commits.len());
    }

    let (hash_to_restore, _) = &commits[selection - 1];
    
    // Safety confirmation
    println!("gini: This will overwrite your current files. Type 'yes' to continue:");
    let mut confirm = String::new();
    std::io::stdin().read_line(&mut confirm)?;
    
    if confirm.trim().to_lowercase() != "yes" {
        println!("gini: Restore cancelled.");
        return Ok(());
    }

    println!("gini: Restoring to checkpoint {}...", hash_to_restore);
    restore(hash_to_restore)?;
    println!("gini: Successfully restored project state.");

    Ok(())
}

/// Restores the project state from a backup using a TUI.
fn restore_backup_tui() -> Result<()> {
    let root_path = find_repo_root()?;
    let backup_dir = root_path.join(".gini/backups");
    
    if !backup_dir.exists() {
        println!("gini: No backups found.");
        return Ok(());
    }
    
    let mut backups = Vec::new();
    for entry in fs::read_dir(&backup_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap().to_str().unwrap();
            if name.starts_with("backup_") {
                backups.push((name.to_string(), path));
            }
        }
    }
    
    if backups.is_empty() {
        println!("gini: No backups found.");
        return Ok(());
    }
    
    // Sort backups by timestamp (newest first)
    backups.sort_by(|a, b| b.0.cmp(&a.0));
    
    // Display available backups
    println!("gini: Available backups:");
    for (i, (name, path)) in backups.iter().enumerate() {
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;
        let datetime: chrono::DateTime<chrono::Local> = chrono::DateTime::from(modified);
        println!("  {}. {} (created: {})", i + 1, name, datetime.format("%Y-%m-%d %H:%M:%S"));
    }

    // Simple text-based selection
    println!("\ngini: Enter backup number to restore (1-{}):", backups.len());
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    
    let selection: usize = input.trim().parse()
        .map_err(|_| anyhow::anyhow!("Invalid selection"))?;
    
    if selection < 1 || selection > backups.len() {
        bail!("Invalid selection: must be between 1 and {}", backups.len());
    }

    let (_, backup_path) = &backups[selection - 1];
    
    // Safety confirmation
    println!("gini: This will overwrite your current files. Type 'yes' to continue:");
    let mut confirm = String::new();
    stdin().read_line(&mut confirm)?;
    
    if confirm.trim().to_lowercase() != "yes" {
        println!("gini: Restore cancelled.");
        return Ok(());
    }

    println!("gini: Restoring from backup {}...", backup_path.file_name().unwrap().to_str().unwrap());
    restore_from_backup(&root_path, backup_path)?;
    println!("gini: Successfully restored from backup.");

    Ok(())
}

fn restore_from_backup(root_path: &Path, backup_path: &Path) -> Result<()> {
    // Clean current working directory (excluding .gini)
    clean_working_directory(root_path)?;
    
    // Copy backup contents to root
    copy_directory_excluding(backup_path, root_path, &[".gini"])?;
    
    Ok(())
}

// --- Core VCS Functions ---

pub fn init() -> Result<()> {
    let gini_path = Path::new(".gini");
    if gini_path.exists() {
        bail!("--- .gini already exists.");
    }
    
    // Create directory structure atomically
    fs::create_dir(gini_path)
        .context("Failed to create .gini directory")?;
    fs::create_dir(gini_path.join("objects"))
        .context("Failed to create objects directory")?;
    fs::create_dir_all(gini_path.join("refs/heads"))
        .context("Failed to create refs directory")?;
    
    // Write HEAD file atomically
    let head_content = "ref: refs/heads/main";
    let head_path = gini_path.join("HEAD");
    write_file_atomic(&head_path, head_content.as_bytes())
        .context("Failed to write HEAD file")?;
    
    println!(
        "gini: Initialized empty .gini project in {}",
        std::env::current_dir()?.display()
    );
    Ok(())
}

pub fn ensure_initialized() -> Result<()> {
    if find_repo_root().is_err() {
        eprintln!("gini: No .gini project found in this directory.\n--- Run `gini init` first.");
        std::process::exit(1);
    }
    Ok(())
}

pub fn checkpoint(message: &str) -> Result<String> {
    let root_path = find_repo_root()?;
    let objects_path = root_path.join(".gini/objects");
    
    // Validate objects directory
    if !objects_path.exists() {
        bail!("Objects directory not found. Repository may be corrupted.");
    }
    
    let tree_hash = write_tree(&root_path, &objects_path)?;
    let parent_hash = get_head_commit(&root_path)?;
    
    // Get author info from environment or use defaults
    let author_name = std::env::var("GINI_AUTHOR_NAME")
        .unwrap_or_else(|_| "Unknown".to_string());
    let author_email = std::env::var("GINI_AUTHOR_EMAIL")
        .unwrap_or_else(|_| "unknown@example.com".to_string());
    
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let parent_line = parent_hash
        .map(|h| format!("parent {}\n", h))
        .unwrap_or_default();

    let commit_content = format!(
        "tree {}\n{}author {} <{}> {} +0530\n\n{}",
        tree_hash, parent_line, author_name, author_email, timestamp, message
    );

    let commit_hash = hash_and_write_object(&objects_path, commit_content.as_bytes())?;
    update_head(&root_path, &commit_hash)?;
    Ok(commit_hash)
}

pub fn restore(commit_hash: &str) -> Result<()> {
    // Validate commit hash
    if !is_valid_hash(commit_hash) {
        bail!("Invalid commit hash: {}", commit_hash);
    }
    
    let root_path = find_repo_root()?;
    let objects_path = root_path.join(".gini/objects");
    
    // Verify commit exists
    let commit_path = objects_path.join(commit_hash);
    if !commit_path.exists() {
        bail!("Commit not found: {}", commit_hash);
    }
    
    let commit_content = read_object(&objects_path, commit_hash)?;
    let tree_hash = parse_commit_tree(&commit_content)?;

    // Create backup before destructive operation
    create_backup(&root_path)?;
    
    clean_working_directory(&root_path)?;
    restore_tree(&root_path, &objects_path, &tree_hash)?;
    update_head(&root_path, commit_hash)?;
    Ok(())
}

pub fn log() -> Result<String> {
    let root_path = find_repo_root()?;
    let mut history = String::new();
    let mut current_commit_hash: Option<String> = get_head_commit(&root_path)?;

    while let Some(hash) = current_commit_hash {
        let commit_content = read_object(&root_path.join(".gini/objects"), &hash)?;
        let (parent, author, message) = parse_commit_details(&commit_content)?;
        history.push_str(&format!(
            "checkpoint {}\nAuthor: {}\n\n\t{}\n\n",
            hash, author, message
        ));
        current_commit_hash = parent;
    }
    Ok(history)
}

pub fn get_commit_history() -> Result<Vec<(String, String)>> {
    let root_path = find_repo_root()?;
    let mut history = Vec::new();
    let mut current_commit_hash: Option<String> = get_head_commit(&root_path)?;

    while let Some(hash) = current_commit_hash {
        let commit_content = read_object(&root_path.join(".gini/objects"), &hash)?;
        let (parent, _, message) = parse_commit_details(&commit_content)?;
        history.push((hash, message.lines().next().unwrap_or("").to_string()));
        current_commit_hash = parent;
    }
    Ok(history)
}

// --- Internal Helper Functions ---

fn find_repo_root() -> Result<PathBuf> {
    let mut current_dir = std::env::current_dir()?;
    let mut depth = 0;
    const MAX_DEPTH: u32 = 100; // Prevent infinite loops
    
    loop {
        if current_dir.join(".gini").is_dir() {
            return Ok(current_dir);
        }
        if !current_dir.pop() || depth >= MAX_DEPTH {
            bail!("Not a Gini repository.");
        }
        depth += 1;
    }
}

fn is_valid_hash(hash: &str) -> bool {
    hash.len() == HASH_LENGTH && hash.chars().all(|c| c.is_ascii_hexdigit())
}

fn write_file_atomic(path: &Path, content: &[u8]) -> Result<()> {
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, content)?;
    fs::rename(temp_path, path)?;
    Ok(())
}

fn create_backup(root_path: &Path) -> Result<()> {
    let backup_dir = root_path.join(".gini/backups");
    fs::create_dir_all(&backup_dir)?;
    
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let backup_path = backup_dir.join(format!("backup_{}", timestamp));
    
    // Copy current state to backup
    copy_directory_excluding(root_path, &backup_path, &[".gini"])?;
    println!("gini: Created backup at {:?}", backup_path);
    Ok(())
}

fn copy_directory_excluding(src: &Path, dst: &Path, exclude: &[&str]) -> Result<()> {
    if src.is_file() {
        fs::copy(src, dst)?;
        return Ok(());
    }
    
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();
        
        if exclude.contains(&name) {
            continue;
        }
        
        let dst_path = dst.join(name);
        if path.is_dir() {
            copy_directory_excluding(&path, &dst_path, exclude)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }
    Ok(())
}

fn hash_and_write_object(objects_path: &Path, content: &[u8]) -> Result<String> {
    // Check file size limit
    if content.len() as u64 > MAX_FILE_SIZE {
        bail!("File too large (max {} bytes)", MAX_FILE_SIZE);
    }
    
    let mut hasher = Sha1::new();
    hasher.update(content);
    let hash_string = hex::encode(hasher.finalize());
    
    // Validate hash format
    if !is_valid_hash(&hash_string) {
        bail!("Generated invalid hash: {}", hash_string);
    }
    
    let object_file_path = objects_path.join(&hash_string);

    if !object_file_path.exists() {
        let temp_path = object_file_path.with_extension("tmp");
        fs::write(&temp_path, content)?;
        fs::rename(temp_path, &object_file_path)?;
    }
    Ok(hash_string)
}

fn read_object(objects_path: &Path, hash: &str) -> Result<String> {
    // Validate hash
    if !is_valid_hash(hash) {
        bail!("Invalid hash format: {}", hash);
    }
    
    let path = objects_path.join(hash);
    if !path.exists() {
        bail!("Object not found: {}", hash);
    }
    
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read object: {}", hash))?;
    Ok(content)
}

fn write_tree(dir_path: &Path, objects_path: &Path) -> Result<String> {
    let mut entries = BTreeMap::new();
    
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

        if [".gini", ".git", "target"].contains(&file_name) {
            continue;
        }

        if path.is_dir() {
            let sub_tree_hash = write_tree(&path, objects_path)?;
            entries.insert(file_name.to_string(), format!("tree {}", sub_tree_hash));
        } else {
            // Check file size before reading
            let metadata = fs::metadata(&path)?;
            if metadata.len() > MAX_FILE_SIZE {
                bail!("File too large: {} (max {} bytes)", path.display(), MAX_FILE_SIZE);
            }
            
            let content = fs::read(&path)?;
            let blob_hash = hash_and_write_object(objects_path, &content)?;
            entries.insert(file_name.to_string(), format!("blob {}", blob_hash));
        }
    }
    
    let tree_content = entries
        .iter()
        .map(|(name, entry)| format!("{}  {}", entry, name))
        .collect::<Vec<_>>()
        .join("\n");
    hash_and_write_object(objects_path, tree_content.as_bytes())
}

fn restore_tree(target_dir: &Path, objects_path: &Path, tree_hash: &str) -> Result<()> {
    if !is_valid_hash(tree_hash) {
        bail!("Invalid tree hash: {}", tree_hash);
    }
    
    let tree_content = read_object(objects_path, tree_hash)?;
    
    for line in tree_content.lines() {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() != 3 {
            bail!("Invalid tree entry format: {}", line);
        }
        
        let (obj_type, hash, name) = (parts[0], parts[1], parts[2]);
        
        // Validate object type
        if obj_type != "tree" && obj_type != "blob" {
            bail!("Invalid object type: {}", obj_type);
        }
        
        // Validate hash
        if !is_valid_hash(hash) {
            bail!("Invalid hash in tree: {}", hash);
        }
        
        // Validate filename
        if name.is_empty() || name.contains('/') || name.contains('\\') {
            bail!("Invalid filename in tree: {}", name);
        }
        
        let path = target_dir.join(name);

        if obj_type == "tree" {
            fs::create_dir_all(&path)?;
            restore_tree(&path, objects_path, hash)?;
        } else {
            let blob_content = read_object_raw(objects_path, hash)?;
            fs::write(path, blob_content)?;
        }
    }
    Ok(())
}

fn read_object_raw(objects_path: &Path, hash: &str) -> Result<Vec<u8>> {
    if !is_valid_hash(hash) {
        bail!("Invalid hash format: {}", hash);
    }
    
    let path = objects_path.join(hash);
    if !path.exists() {
        bail!("Object not found: {}", hash);
    }
    
    let content = fs::read(&path)
        .with_context(|| format!("Failed to read object: {}", hash))?;
    Ok(content)
}

fn clean_working_directory(root_path: &Path) -> Result<()> {
    for entry in fs::read_dir(root_path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;
            
        if file_name != ".gini" && file_name != ".git" {
            if path.is_dir() {
                fs::remove_dir_all(&path)?;
            } else {
                fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}

fn get_head_commit(root_path: &Path) -> Result<Option<String>> {
    let head_path = root_path.join(".gini/HEAD");
    if !head_path.exists() {
        return Ok(None);
    }
    
    let head_content = fs::read_to_string(&head_path)?;
    if let Some(ref_path_str) = head_content.strip_prefix("ref: ") {
        let ref_path = root_path.join(".gini").join(ref_path_str.trim());
        if ref_path.exists() {
            let content = fs::read_to_string(&ref_path)?;
            let hash = content.trim();
            if is_valid_hash(hash) {
                Ok(Some(hash.to_string()))
            } else {
                bail!("Invalid hash in ref file: {}", hash);
            }
        } else {
            Ok(None)
        }
    } else if head_content.len() == HASH_LENGTH {
        let hash = head_content.trim();
        if is_valid_hash(hash) {
            Ok(Some(hash.to_string()))
        } else {
            bail!("Invalid hash in HEAD: {}", hash);
        }
    } else {
        bail!("Invalid HEAD format")
    }
}

fn update_head(root_path: &Path, commit_hash: &str) -> Result<()> {
    if !is_valid_hash(commit_hash) {
        bail!("Invalid commit hash: {}", commit_hash);
    }
    
    let head_path = root_path.join(".gini/HEAD");
    let head_content = fs::read_to_string(&head_path)?;
    let ref_path_str = head_content
        .strip_prefix("ref: ")
        .ok_or_else(|| anyhow::anyhow!("Detached HEAD not supported for updates"))?;
    let ref_path = root_path.join(".gini").join(ref_path_str.trim());
    
    // Write atomically
    write_file_atomic(&ref_path, commit_hash.as_bytes())?;
    Ok(())
}

fn parse_commit_tree(commit_content: &str) -> Result<String> {
    let tree_line = commit_content
        .lines()
        .find(|line| line.starts_with("tree "))
        .ok_or_else(|| anyhow::anyhow!("Could not find tree in commit object"))?;
    
    let parts: Vec<_> = tree_line.split_whitespace().collect();
    if parts.len() != 2 {
        bail!("Invalid tree line format: {}", tree_line);
    }
    
    let hash = parts[1];
    if !is_valid_hash(hash) {
        bail!("Invalid tree hash in commit: {}", hash);
    }
    
    Ok(hash.to_string())
}

fn parse_commit_details(commit_content: &str) -> Result<(Option<String>, String, String)> {
    let mut parent = None;
    let mut author = String::new();
    let mut message_lines = Vec::new();
    let mut in_message = false;

    for line in commit_content.lines() {
        if in_message {
            message_lines.push(line);
            continue;
        }
        if line.starts_with("parent ") {
            let parts: Vec<_> = line.split_whitespace().collect();
            if parts.len() == 2 && is_valid_hash(parts[1]) {
                parent = Some(parts[1].to_string());
            } else {
                bail!("Invalid parent line: {}", line);
            }
        } else if line.starts_with("author ") {
            author = line.strip_prefix("author ").unwrap().to_string();
        } else if line.is_empty() {
            in_message = true;
        }
    }
    Ok((parent, author, message_lines.join("\n")))
}
