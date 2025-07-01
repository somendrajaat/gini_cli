# gini

A simple command-line checkpoint system for your projects. `gini` lets you create, list, and restore checkpoints in your project directory, making it easy to save and roll back to different states. It's like having a mini, local version control system for quick snapshots.

## Features

- **Initialize**: Set up `gini` in your project with a single command.
- **Create**: Make a named checkpoint of your current project state (using regular file copies).
- **Restore**: Roll back all your project files to a previously saved checkpoint.
- **List**: View all the checkpoints you've created.
- **Delete**: Remove checkpoints you no longer need.

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

To save a snapshot of your project, create a checkpoint with a descriptive name.

```bash
gini --checkpoint "my-first-checkpoint"
```
You can also use the shorter `-c` flag:
```bash
gini -c "refactoring-done"
```
This saves the current state of your files (excluding `.gini`, `.git`, and `target`) into a new checkpoint. Each checkpoint is a full, independent copy of your project at that moment. Files are copied, not hard-linked, so changes in your working directory do not affect previous checkpoints.

### 3. List Available Checkpoints

To see a list of all the checkpoints you've saved:

```bash
gini --list
```
Or with the `-l` flag:
```bash
gini -l
```

### 4. Restore a Checkpoint

If you need to revert your project to a previous state, use the restore command. You will be prompted to select a checkpoint interactively:

```bash
gini --restore
```
Or with the `-r` flag:
```bash
gini -r
```
This will replace your current files with the files from the selected checkpoint. **Warning:** This will overwrite existing files in your project directory.

### 5. Delete a Checkpoint

To delete a checkpoint interactively:

```bash
gini --delete
```
Or with the `-d` flag:
```bash
gini -d
```

## Notes and Limitations

- **Full Copies:** Each checkpoint is a full copy of your project files. Changes in your working directory after creating a checkpoint do not affect previous checkpoints, and vice versa.
- **Exclusions:** By default, `.gini`, `.git`, and `target` directories are excluded from checkpoints.
- **Symlinks:** Symlinks are not handled specially and may be copied as regular files or skipped.
- **Overwrite Warning:** Restoring a checkpoint will overwrite files in your project directory. Make sure to commit or back up important changes before restoring.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details. 