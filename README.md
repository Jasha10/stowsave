# StowSave

`stowsave` automates the process of moving files from their original location to a Stow package
directory then running GNU stow to create symlinks. Backups of the original files are created
to prevent data loss.

For example, say your working on `~/dev/project/my_script`. Invoking `stowsave` as

```
stowsave ~/dev/project/my_script ~/my/stow/directory
```

will do the following:
  1. create a backup of `~/dev/project/my_script` at `~/dev/project/my_script.bak`,
  2. move `~/dev/project/my_script` to `~/my/stow/directory/dev/project/my_script`, and
  3. run `stow` in `~/my/stow/directory` to create a symlink at `~/dev/project/my_script`
     pointing to `~/my/stow/directory/dev/project/my_script`.

## Usage
```
stowsave <PATH_TO_SAVE> <STOW_PACKAGE>
```
- `<PATH_TO_SAVE>`: The path to the file or directory you want to save
- `<STOW_PACKAGE>`: The directory where your Stow packages are stored

What does the above do?
- Creates a backup of the given <PATH_TO_SAVE> file or directory, backing up to <PATH_TO_SAVE>.bak.
    - For directories, the backup is recursive
- Move the original <PATH_TO_SAVE> to the given <STOW_PACKAGE>
- Run `stow` to create symlinks from the <STOW_PACKAGE> to the original location of <PATH_TO_SAVE>.

## Example
```
stowsave ~/.vimrc ~/dotfiles/vim
```
This command will:
1. Copy `~/.vimrc` to `~/dotfiles/vim/.vimrc`
2. Rename the original `~/.vimrc` to `~/.vimrc.bak`
3. Run `stow vim` in the `~/dotfiles` directory

## Requirements

- Rust (for building)
- GNU Stow

##
This README file is generated based on the docs in `src/main.rs`.
