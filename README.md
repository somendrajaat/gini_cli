# gini

A simple command-line checkpoint system for your projects. `gini` lets you create, list, and restore checkpoints in your project directory, making it easy to save and roll back to different states. It's like having a mini, local version control system for quick snapshots.

## Features

- **Initialize**: Set up `gini` in your project with a single command.
- **Create**: Make a named checkpoint of your current project state.
- **Restore**: Roll back all your project files to a previously saved checkpoint.
- **List**: View all the checkpoints you've created.
- **Git Integration**: Automatically stashes uncommitted changes when creating a checkpoint, keeping your Git history clean.

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
This command creates a `.undoit` directory where all your checkpoints will be stored.

### 2. Create a Checkpoint

To save a snapshot of your project, create a checkpoint with a descriptive name.

```bash
gini --checkpoint "my-first-checkpoint"
```
You can also use the shorter `-c` flag:
```bash
gini -c "refactoring-done"
```
This saves the current state of your files (excluding `.undoit` and `.git`) into a new checkpoint.

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

If you need to revert your project to a previous state, use the restore command with the checkpoint name.

```bash
gini --restore "my-first-checkpoint"
```
Or with the `-r` flag:
```bash
gini -r "my-first-checkpoint"
```
This will replace your current files with the files from the specified checkpoint.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details. 