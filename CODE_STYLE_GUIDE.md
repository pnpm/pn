# Code Style Guide

## Introduction

Clippy cannot yet detect all suboptimal code. This guide supplements that.

This guide is incomplete. More may be added as more pull requests are going to be reviewed.

This is a guide, not a rule. Contributors may break them if they have a good reason to do so.

## Terminology

[owned]: #owned-type
[borrowed]: #borrowed-type
[copying]: #copying

### Owned type

Doesn't have a lifetime, neither implicit nor explicit.

*Examples:* `String`, `OsString`, `PathBuf`, `Vec<T>`, etc.

### Borrowed type

Has a lifetime, either implicit or explicit.

*Examples:* `&str`, `&OsStr`, `&Path`, `&[T]`, etc.

### Copying

The act of cloning or creating an owned data from another owned/borrowed data.

*Examples:*
* `x.clone()`
* `borrowed_data.to_owned()`
* `OwnedType::from(borrowed_data)`
* `path.to_path_buf()`
* `str.to_string()`
* etc.

## Guides

### When to use [owned] parameter? When to use [borrowed] parameter?

This is a trade-off between API flexibility and performance.

If using an [owned] signature would reduce [copying], one should use an [owned] signature.

Otherwise, use a [borrowed] signature to widen the API surface.

**Example 1:** Preferring [owned] signature.

```rust
fn push_path(mut list: Vec<PathBuf>, item: &Path) {
    list.push(item.to_path_buf());
}

push_path(my_list, my_path_buf);
push_path(my_list, my_path_ref.to_path_buf());
```

The above code is suboptimal because it forces the [copying] of `my_path_buf` even though the type of `my_path_buf` is already `PathBuf`.

Changing the signature of `item` to `PathBuf` would help remove `.to_path_buf()` inside the `push_back` function, eliminate the cloning of `my_path_buf` (the ownership of `my_path_buf` is transferred to `push_path`).

```rust
fn push_path(mut list: Vec<PathBuf>, item: PathBuf) {
    list.push(item);
}

push_path(my_list, my_path_buf);
push_path(my_list, my_path_ref.to_path_buf());
```

It does force `my_path_ref` to be explicitly copied, but since `item` is not copied, the total number of copying remains the same for `my_path_ref`.

**Example 2:** Preferring [borrowed] signature.

```rust
fn show_path(path: PathBuf) {
    println!("The path is {path:?}");
}

show_path(my_path_buf);
show_path(my_path_ref.to_path_buf());
```

The above code is suboptimal because it forces the [copying] of `my_path_ref` even though a `&Path` is already compatible with the code inside the function.

Changing the signature of `path` to `&Path` would help remove `.to_path_buf()`, eliminating the unnecessary copying:

```rust
fn show_path(path: &Path) {
    println!("The path is {path:?}");
}

show_path(my_path_buf);
show_path(my_path_ref);
```

### Use the most encompassing type for function parameters

The goal is to allow the function to accept more types of parameters, reducing type conversion.

**Example 1:**

```rust
fn node_bin_dir(workspace: &PathBuf) -> PathBuf {
    workspace.join("node_modules").join(".bin")
}

let a = node_bin_dir(&my_path_buf);
let b = node_bin_dir(&my_path_ref.to_path_buf());
```

The above code is suboptimal because it forces the [copying] of `my_path_ref` only to be used as a reference.

Changing the signature of `workspace` to `&Path` would help remove `.to_path_buf()`, eliminating the unnecessary copying:

```rust
fn node_bin_dir(workspace: &PathBuf) -> PathBuf {
    workspace.join("node_modules").join(".bin")
}

let a = node_bin_dir(&my_path_buf);
let b = node_bin_dir(my_path_ref);
```
