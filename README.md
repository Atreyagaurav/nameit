CLI to rename your files based on the metadata or user inputs

Slowly building this. I think it'll be fun and usable, no need to rename files in the same templates again and again.

# Features
- Uses the template system for different parts in a name,
- Remembers the user inputs, so you can reuse the previous parts,
- Just rename, or copy (default) the current file,
- Can use numbering system, or reuse parts of the previous name
# Installation
Clone the repo, and run `cargo build --release`.

For arch users,
- Get it from [AUR](https://aur.archlinux.org/packages/nameit), OR 
- simply download the `PKGBUILD` file and then do `makepkg` and `makepkg --install`.

# Usage
Run the command `nameit` with filename as argument to rename. Provide the choices for format, and then variables in that format. Use `_` to separate the variables in the format. For example, format `NAME_VER` will use two variables `NAME` and `VER`, you can give inputs to those variables. It'll remember your inputs and save it for later use. 

When you have choices, enter the choice number to choose it, otherwise enter 0, and it'll give you the option to enter a new entry, it'll save that entry to the history.

# Editing the Saved choices
you can run `nameit -e` to run an interactive session to filter the saved choices. 

You can filter the formats, (remember that if you remove a format and there are variables only used in that format, you can remove them by entering 0 for the choices to filter), you can filter the choices for the variables. Press enter with no inputs to just leave it be, otherwise, use `start-end` format that'll only keep the choices in that range (inclusive). You can just use `-end` or `start-` format, if you want to just denote the lower and upper limit only. For example, `1-5` will keep entries 1 to 5, and remove everything else, while `-5` also has the same effect, and something like `3-` will keep everything from 3 onwards and only remove 1 and 2.

# Special Template Variables
## Numbering
Any variable with a multiple `#` character is considered a number format. It'll be rendered as loop index for the file being processed that starts with 1 and is zero padded. For example `###` will start from `001`.
## Old Filename Parts
If you use `*` in the format, it will use the first part of the old filename, more * you have more parts it'll reuse. Parts are defined as the strings separated by `_`. You can use `?` to include the whole previous filename.

# TODO 
- [x] support user inputs
- [x] save user inputs for later reuse
- [ ] threshold for user inputs
- [x] Copy or Replace current file flag (destination directory for copy)
- [x] Batch Process
- [ ] Replace or countup flag for same name conflicts
- [ ] Single Keypress if possible for input
- [x] Add colors for different kind of input (choice+manual)
- [ ] support file metadata extration for variables
- [ ] compile for windows
