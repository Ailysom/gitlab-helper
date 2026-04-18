# gitlab_helper

A small CLI for reading and cloning GitLab groups and projects.

## Configuration

Authentication and target instance are supplied via flags or environment variables:

| Flag      | Env var         | Default              |
| --------- | --------------- | -------------------- |
| `--token` | `GITLAB_TOKEN`  | *(required)*         |
| `--url`   | `GITLAB_URL`    | `https://gitlab.com` |

The token needs `read_api` (for `list`/`get`) and `read_repository` (for `clone`). Run `gitlab_helper --help` for the full list of options.

## Syntax

```
gitlab_helper [-n <group>] [-r] <command> <object_type> [<name>]
```

**Commands**

| Command | Behavior                                                                 |
| ------- | ------------------------------------------------------------------------ |
| `list`  | Print objects of the given type.                                         |
| `get`   | Fetch metadata for the specified object. For groups, also print a child tree. |
| `clone` | Clone repo(s); when applied to a group, clone every repo it contains.    |

**Object types:** `group`, `repo`.

**Flags**

- `-n <group>` — set the operation context to `<group>`. Omit to operate at the root.
- `-r` — recurse into subgroups.
- `--http` — clone over HTTPS instead of SSH.

**Trailing `<name>`** addresses a specific object within the context. For example, `get repo my-repo` with `-n my_group` fetches metadata for `my_group/my-repo`.

## Examples

```sh
# List repos directly inside my_group
gitlab_helper -n my_group list repo

# List all repos under my_group at any depth (full paths)
gitlab_helper -n my_group -r list repo

# List every group in the instance as full paths
gitlab_helper -r list group

# Fetch metadata + a child tree for my_group
gitlab_helper -n my_group -r get group

# Fetch metadata for a single repo
gitlab_helper -n my_group get repo my-repo

# Clone every repo directly inside my_group, preserving hierarchy
gitlab_helper -n my_group clone group

# Clone every repo under my_group at any depth
gitlab_helper -n my_group -r clone group
```

## Output behavior

`list` without `-r` prints the path segment of each direct child (e.g. `my-repo`). With `-r`, every match is printed as its full path (e.g. `my_group/sub_group/my-repo`).

`get group` prints metadata, then a tree of children. `-r` expands the tree to all descendants; without it, only direct children are shown. Subgroups are suffixed with `/`.

```
my_group/
├── group1/
│   ├── repo1
│   └── repo2
└── repo3
```

## Clone behavior

Clone preserves the group hierarchy **relative to the queried context**. The queried group itself does not appear in the output directory — its contents are cloned directly into the current working directory.

Given this structure under `my_group`:

```
my_group/
├── repo3
└── group1/
    ├── repo1
    └── repo2
```

`gitlab_helper -n my_group -r clone group` produces:

```
./
├── repo3/
└── group1/
    ├── repo1/
    └── repo2/
```
