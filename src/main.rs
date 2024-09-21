//! # StowSave
//!
//! This command line utility copies your precious dotfiles (or others) into a local stow directory
//! then drives GNU stow to create symlinks to to those dotfiles.
//!
//! For example, say your working on `~/dev/project/my_script`. You can invoke
//!
//! ```
//! stowsave ~/dev/project/my_script ~/my/stow/directory
//! ```
//!
//! to move `my_script` to (1) move `~/dev/project/my_script` to `~/my_stow_directory/
//! StowSave is a command-line utility that helps manage dotfiles and other configuration files using
//! GNU Stow. It automates the process of moving files from their original location to a Stow package
//! directory, then running Stow to create symlinks. Backups of the original files are created to
//! prevent data loss.
//!
//! ## Usage
//! ```
//! stowsave <PATH_TO_SAVE> <STOW_DIRECTORY>
//! ```
//! - `<PATH_TO_SAVE>`: The path to the file or directory you want to save
//! - `<STOW_DIRECTORY>`: The directory where your Stow packages are stored
//!
//! What does the above do?
//! - Creates a backup of the given <PATH_TO_SAVE> file or directory, backing up to <PATH_TO_SAVE>.bak.
//!     - For directories, the backup is recursive
//! - Move the original <PATH_TO_SAVE> to the given <STOW_DIRECTORY>
//! - Run `stow` to create symlinks from the <STOW_DIRECTORY> to the original location of <PATH_TO_SAVE>.
//!
//! ## Example
//! ```
//! stowsave ~/.vimrc ~/dotfiles/vim
//! ```
//! This command will:
//! 1. Copy `~/.vimrc` to `~/dotfiles/vim/.vimrc`
//! 2. Rename the original `~/.vimrc` to `~/.vimrc.bak`
//! 3. Run `stow vim` in the `~/dotfiles` directory
//!
//! ## Requirements
//!
//! - Rust (for building)
//! - GNU Stow

mod checks;
mod command_impl;
mod util;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use command_impl::CommandImpl;
use util::find_common_ancestor;

#[derive(Debug)]
enum Command {
    CreateDirIfNotExists(PathBuf),
    MoveToDir {
        from: PathBuf,
        dir: PathBuf,
    },
    CreateBackup {
        original: PathBuf,
        backup_name: String,
    },
    RunStow {
        pwd: PathBuf,
        package: String,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the file or directory to save
    path_to_save: PathBuf,

    /// Directory where stow packages are stored
    stow_directory: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let commands = collect_commands(&args)?;

    execute_commands(commands, args.verbose)?;

    // TODO:
    // checks::check_that_symlink_has_been_created(&args.path_to_save, &args.stow_directory)?;

    println!("Path successfully saved, backed up, and stowed");
    Ok(())
}

fn backup_path_command(original: &Path) -> Command {
    let backup_name: String = original.file_name().unwrap().to_str().unwrap().to_string() + ".bak";
    Command::CreateBackup {
        original: original.to_owned(),
        backup_name,
    }
}

fn collect_commands(args: &Args) -> Result<Vec<Command>> {
    let mut commands = Vec::new();

    let path_to_save = args
        .path_to_save
        .canonicalize()
        .context("Failed to canonicalize path_to_save")?;

    commands.push(backup_path_command(&path_to_save));

    let stow_dir = args
        .stow_directory
        .canonicalize()
        .context("Failed to canonicalize stow_directory")?;

    checks::path_to_save_exists(&path_to_save)?;
    checks::path_to_save_is_not_symlink(&path_to_save)?;
    checks::stow_directory_exists(&stow_dir)?;

    let common_ancestor = find_common_ancestor(&path_to_save, &stow_dir);
    checks::stow_directory_is_grandchild_of_common_ancestor(&stow_dir, &common_ancestor)?;

    // Compute the relative path from the common ancestor to the path to save
    let relative_path_from_ancestor_to_path_to_save =
        path_to_save.strip_prefix(&common_ancestor)?;

    let target_path = stow_dir.join(relative_path_from_ancestor_to_path_to_save);
    checks::target_path_does_not_exist(&target_path)?;
    let target_dir = target_path.parent().unwrap().to_owned();
    commands.push(Command::CreateDirIfNotExists(target_dir.clone()));

    commands.push(Command::MoveToDir {
        from: path_to_save.clone(),
        dir: target_dir,
    });

    let stow_package = stow_dir.file_name().unwrap().to_str().unwrap().to_string();
    commands.push(Command::RunStow {
        pwd: stow_dir.parent().unwrap().to_owned(),
        package: stow_package,
    });

    Ok(commands)
}

fn execute_commands(commands: Vec<Command>, verbose: bool) -> Result<()> {
    for command in commands {
        command.invoke(verbose)?;
    }
    Ok(())
}
