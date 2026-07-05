# tasks CLI

A terminal client for the tasks daemon. Explores and edits `.tasks` workspaces
the same way the desktop app does: every command maps onto one daemon route
(see [HTTP API](../../docs/http-api.md)), so no core logic lives here.

The package is `tasks-cli`; the binary is **`tasks`**.

## Requirements

The daemon must be running — the CLI is a client, not a standalone tool:

```sh
cargo run -p tasks-server
```

## Running

```sh
cargo run -p tasks-cli -- --help    # via cargo
cargo install --path apps/cli       # or install the `tasks` binary
tasks --help
```

## Workspace resolution

Commands operate on a workspace (a repo with a `.tasks/` directory). Like
git's repo discovery, the CLI walks up from the current directory until it
finds one — run it from anywhere inside a repo and it just works:

```sh
cd ~/code/my-repo/src/deeply/nested
tasks task list                     # operates on ~/code/my-repo
```

An explicit `--workspace <PATH>` overrides discovery. A directory with no
`.tasks` ancestor is an error — initialize one first with `tasks init`.

## Global flags

| Flag | Effect |
|---|---|
| `--workspace <PATH>` | Workspace root; defaults to the nearest ancestor containing `.tasks`. |
| `--daemon <URL>` | Daemon base URL (also via `TASKS_DAEMON` env). Default `http://127.0.0.1:4000`. |
| `--json` | Print the daemon's raw JSON responses instead of human-readable output. |

## Commands

### Workspace

```sh
tasks init [PATH]        # create a .tasks workspace (defaults to the current directory)
tasks health             # check that the daemon is reachable
```

### Projects

```sh
tasks project list                    # open projects, oldest first
tasks project list --closed           # closed (archived) projects instead
tasks project show <ID>
tasks project create "Q3 Roadmap"
tasks project rename <ID> "New name"
tasks project close <ID>
tasks project reopen <ID>
```

### Tasks

```sh
tasks task list                                # open tasks, oldest first
tasks task list --project <PROJECT_ID>         # scoped to one project
tasks task list --closed                       # closed tasks instead
tasks task show <ID>
tasks task history <ID>                        # full event history, oldest first
tasks task create "Ship it" --project <PROJECT_ID>
tasks task rename <ID> "New name"
tasks task move <ID> <PROJECT_ID>              # re-parent to another project
tasks task close <ID>
tasks task reopen <ID>
tasks task status <ID> <STATUS_ID>             # set the workflow status
tasks task status <ID> --clear                 # clear it
tasks task describe <ID> "Markdown body..."    # replace the description
```

`task list` resolves status and project IDs to names (two extra local
requests), so exploring reads like the app, not like raw storage. `task
history` shows the task's event log as `TIME  EVENT  DETAILS` rows — the
event-sourced audit trail, straight from disk.

### Statuses

```sh
tasks status list                              # canonical (seeds-first) order
tasks status list --removed                    # soft-removed statuses instead
tasks status create "Blocked" --kind started [--description "..."]
tasks status rename <ID> "New name"
tasks status describe <ID> "..."               # update the description
tasks status describe <ID> --clear             # clear it
tasks status kind <ID> complete                # change the semantic kind
tasks status remove <ID>                       # soft-remove
```

`--kind` takes one of the semantic kinds: `unstarted`, `started`, `complete`,
`canceled` (see [Data Model](../../docs/data-model.md)). A fresh workspace
already has the six built-in seed statuses — see [Seeds](../../docs/seeds.md).

## Scripting with `--json`

`--json` swaps the rendering for the daemon's raw responses, one JSON document
per invocation — pipe it to `jq`:

```sh
# IDs of all open tasks in a project
tasks --json task list --project "$PROJECT" | jq -r '.[].id'

# Create a task and capture its ID
TASK=$(tasks --json task create "Ship it" --project "$PROJECT" | jq -r '.id')

# Close every open task in a project
tasks --json task list --project "$PROJECT" | jq -r '.[].id' \
  | xargs -I{} tasks task close {}
```

Writes return the freshly rebuilt entity, so there's no need for a follow-up
read. Errors print to stderr and exit non-zero.
