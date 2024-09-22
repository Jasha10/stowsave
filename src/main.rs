//! # StowSave
//!
//! `stowsave` automates the process of moving files from their original location to a Stow package
//! directory then running GNU stow to create symlinks. Backups of the original files are created
//! to prevent data loss.
//!
//! For example, say your working on `~/dev/project/my_script`. Invoking `stowsave` as
//!
//! ```
//! stowsave ~/dev/project/my_script ~/my/stow/directory
//! ```
//!
//! will do the following:
//!   1. create a backup of `~/dev/project/my_script` at `~/dev/project/my_script.bak`,
//!   2. move `~/dev/project/my_script` to `~/my/stow/directory/dev/project/my_script`, and
//!   3. run `stow` in `~/my/stow/directory` to create a symlink at `~/dev/project/my_script`
//!      pointing to `~/my/stow/directory/dev/project/my_script`.
//!
//! ## Installation
//!
//! `cargo install stowsave`
//!
//! ## Usage
//! ```
//! stowsave <PATH_TO_SAVE> <STOW_PACKAGE>
//! ```
//! - `<PATH_TO_SAVE>`: The path to the file or directory you want to save
//! - `<STOW_PACKAGE>`: The directory where your Stow packages are stored
//!
//! What does the above do?
//! - Creates a backup of the given `<PATH_TO_SAVE>` file or directory, backing up to
//! `<PATH_TO_SAVE>.bak`. For directories, the backup is recursive is a recursive copy operation.
//! - Move the original `<PATH_TO_SAVE>` to the given `<STOW_PACKAGE>`.
//! - Run `stow` to create symlinks from the `<STOW_PACKAGE>` to the original location of
//! `<PATH_TO_SAVE>`.
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
//!
//! ## Roadmap
//!
//! - [ ] improve tests
//!   - [ ] end-to-end tests
//!   - [ ] create DSL for setup and and verification of directory structures with nested files and symlinks
//! - [ ] add command line flags such as --no-backup
//! - [ ] add a command to undo stowsave. This should reverse the `stowsave` operation:
//!         - remove the symlink that points into the stow package. This could possibly be done by
//!         running `stow -D` on the stow package, but that would remove all symlinks into the
//!         package so we'd have to run `stow` again on the package later to restore the other
//!         symlinks.
//!         - move the original file out of the stow package and to the location where the symlink
//!         was.
//!         - remove empty directorie(s) within the stow package that contained the file that's
//!         been moved.
//! - [ ] ask for confirmation if it seems that the user is going to stow a directory that contains
//! symlinks that point into the current or another stow package. For example, running
//! ```
//! stowsave .config ~/dotfiles/pkg
//! ```
//! might be unwanted if the `.config` folder already contains some symlinks that point into the

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use command::{Command, CommandImpl};
use util::find_common_ancestor;

mod checks;
mod command;
mod util;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the file or directory to save
    path_to_save: PathBuf,

    /// The stow package where the file or directory will be saved
    stow_package: PathBuf,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let commands = collect_commands(&args)?;

    execute_commands(commands, args.verbose)?;

    // TODO:
    // checks::check_that_symlink_has_been_created(&args.path_to_save, &args.stow_package)?;

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

    let stow_pkg = args
        .stow_package
        .canonicalize()
        .context("Failed to canonicalize stow_package")?;

    checks::path_to_save_exists(&path_to_save)?;
    checks::path_to_save_is_not_symlink(&path_to_save)?;
    checks::stow_directory_exists(&stow_pkg)?;

    let common_ancestor = find_common_ancestor(&path_to_save, &stow_pkg);
    checks::stow_directory_is_grandchild_of_common_ancestor(&stow_pkg, &common_ancestor)?;

    // Compute the relative path from the common ancestor to the path to save
    let relative_path_from_ancestor_to_path_to_save =
        path_to_save.strip_prefix(&common_ancestor)?;

    let target_path = stow_pkg.join(relative_path_from_ancestor_to_path_to_save);
    checks::target_path_does_not_exist(&target_path)?;
    let target_dir = target_path.parent().unwrap().to_owned();
    commands.push(Command::CreateDirIfNotExists(target_dir.clone()));

    commands.push(Command::MoveToDir {
        from: path_to_save.clone(),
        dest_dir: target_dir,
    });

    let stow_package = stow_pkg.file_name().unwrap().to_str().unwrap().to_string();
    commands.push(Command::RunStow {
        pwd: stow_pkg.parent().unwrap().to_owned(),
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
