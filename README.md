
# gini checkpoint system

A simple, fast, and secure command-line checkpoint system for your projects. **gini** lets you create, list, and restore checkpoints with the efficiency of Git's snapshot model, making it easy to save and roll back to different states without duplicating data.

It's a lightweight but powerful tool for managing project states with confidence. 🚀

---

## Features

* **⚡ Efficient Snapshots**: Instead of making full copies, `gini` uses a content-addressed storage model inspired by Git. It only stores unique file contents (blobs), saving significant disk space.

* **🌱 Initialize**: Set up `gini` in your project with a single command.

* **📸 Create Checkpoints**: Instantly save a snapshot of your project's state with a descriptive message.

* **⏪ Restore Checkpoints**: Safely roll back your entire project to any previous checkpoint.

* **📜 View History**: See a clean, chronological log of all your checkpoints.

* **🛡️ Automatic Backups**: Before any destructive operation like a restore, `gini` automatically creates a backup, giving you a complete safety net.

* **🔒 Enhanced Security**: Protects against common issues with input validation, path traversal protection, and file size limits.

---

## Installation

You can install `gini` directly from crates.io using Cargo:
```bash
cargo install gini
```

Or, you can build from source:

```bash
git clone https://github.com/somendrajaat/gini_cli.git
cd gini_cli
cargo install --path .
```

## Usage

Here's how to use `gini`:

### 1. Initialize `gini` in Your Project

To start using `gini`, you first need to initialize it in your project's root directory.

```bash
gini init
```
This command creates a `.gini/checkpoints` directory where all your checkpoints will be stored.

### 2. Create a Checkpoint

To save a snapshot of your project, create a checkpoint with a descriptive message.

```bash
gini checkpoint -m "my-first-checkpoint"
```
You can also use the shorter command:
```bash
gini c -m "refactoring-done"
```
This saves the current state of your files (excluding `.gini`, `.git`, and `target`) into a new checkpoint. Each checkpoint is a full, independent copy of your project at that moment.

### 3. List Available Checkpoints

To see a list of all the checkpoints you've saved:

```bash
gini log
```
Or use the shorter command:
```bash
gini l
```

### 4. Restore a Checkpoint

If you need to revert your project to a previous state, use the restore command. You will be prompted to select a checkpoint interactively:

```bash
gini restore
```
Or use the shorter command:
```bash
gini r
```
This will replace your current files with the files from the selected checkpoint. **Warning:** This will overwrite existing files in your project directory.

### 5. Restore from a Backup

If you need to restore from a backup (created automatically before each restore operation), use the backup command:

```bash
gini backup
```
Or use the shorter command:
```bash
gini b
```
This will show you all available backups with their creation timestamps and allow you to restore from any of them.

## How It Works
- gini is built on the same principles as Git. Instead of copying your entire project for each checkpoint, it uses a content-addressed object store.

- **Blobs**: The content of each file is hashed and stored as a "blob." If a file doesn't change, its blob is reused across multiple checkpoints.

- **Trees**: The directory structure is stored in "tree" objects, which point to blobs (files) and other trees (subdirectories).

- **Commits**: A "commit" (or checkpoint) is a snapshot that points to a single top-level tree, along with metadata like the author and your commit message.

This model is incredibly efficient, ensuring that you only store what has changed, which saves both time and disk space.
## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details. 