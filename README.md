# tasks

> A git-native task manager that lives inside your repository.

---

## What is tasks?

tasks is built for developers who want project management versioned alongside their code. No external services, no accounts, no sync issues. Everything lives in the repo.

Built with [Tauri](https://tauri.app), Rust backend, modern frontend.

---

## Core Design Principles

### Event Sourced
Nothing is ever mutated. Every action (creating a task, renaming a project, closing an issue) is written as an immutable event. State is rebuilt by replaying the history, which gives you a full audit trail and the ability to reconstruct your board at any past commit.

### Conflict Free
Events are append-only and every event is its own uniquely named file. Two contributors working at the same time will never produce a git conflict through normal use.

### Repo Native
Everything commits with your code. Branches, merges, history. Works offline, works air-gapped.

---

## File Structure

```
.tasks/
  projects/
    project_<uuidv7>/
      events/
        <uuidv7>.json
  tasks/
    task_<uuidv7>/
      events/
        <uuidv7>.json
```

Projects and tasks sit as flat sibling folders. The relationship between a task and a project lives in the event payload, not the folder structure, so tasks can move between projects without touching the file tree.

---

## Event Format

Every event is a small JSON file named by its UUIDv7 ID:

```json
{
  "id": "01932d8b2a1c4f3e...",
  "type": "created",
  "payload": {
    "project_id": "project_01932b4a7f3e...",
    "title": "Build event store"
  }
}
```

Event types include: `created`, `renamed`, `assigned`, `moved`, `closed`, `reopened`, and more.

---

## IDs

All entities use **UUIDv7**, a UUID format with a millisecond Unix timestamp in the most significant bits. Files sort lexicographically in chronological order with no extra metadata needed.

IDs are prefixed by entity type:

```
project_01932b4a7f3e8c2d...   <- project
task_01932d8b2a1c4f3e...   <- task
```

---

## Why Not GitHub Issues?

| | GitHub Issues | tasks |
|---|---|---|
| Lives in repo | ✗ | ✓ |
| Works offline | ✗ | ✓ |
| Branches with code | ✗ | ✓ |
| Full history in git | ✗ | ✓ |
| No external account needed | ✗ | ✓ |
| Conflict free collaboration | ✗ | ✓ |

---

## Status

🚧 Early development