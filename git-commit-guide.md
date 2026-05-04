# Git Commit Messages Guide

> Project-specific conventions for slideshow-stack

---

## The 7 Golden Rules

1. Separate subject from body with a blank line
2. Limit subject to 50 characters (hard limit: 72)
3. Capitalize the subject line
4. Do not end with a period
5. Use imperative mood: "If applied, this commit will **your subject**"
6. Wrap body at 72 characters
7. Explain *what and why*, not *how*

### Imperative Mood

Git itself uses imperative:

```
Merge branch 'myfeature'
Revert "Add the thing"
```

- If applied, this commit will **refactor slideshow-rs renderer**
- If applied, this commit will **remove deprecated methods**

---

## Conventional Commits

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Commit Types

feat | fix | refactor | perf | build | chore

### Project Scopes

| Scope | Description |
|-------|-------------|
| store | apps/store |
| slideshow-rs | libs/slideshow-rs |
| video-rs | libs/video-rs |
| raylib-rs | libs/raylib-rs |
| ffmpeg-rs | libs/ffmpeg-rs |
| deploy | Deployment |
| flake | Nix flake |
| compose | Docker/Podman |
| quadlet | Podman quadlets |
| systemd | systemd services |

### Examples

```
feat(store): add slideshow upload endpoint
fix(slideshow-rs): handle empty image queue
refactor(video-rs): simplify frame buffer allocation
build(flake): update Rust to 1.93.0
```

---

## Breaking Changes

The `!` syntax and `BREAKING CHANGE:` footer are equivalent:

```
feat(store)!: change API response format
```

```
feat(store): migrate to new media service

BREAKING CHANGE: MediaService API signature changed
```

---

## Commit Body

Answer three questions:

1. **What is wrong?** The problem this change solves.
2. **Why this solution?** Justify your approach.
3. **What alternatives existed?** Mention discarded alternatives.

Write problem statements in present tense:
- The code does X when given Y input

### What and Why, Not How

The code shows *how*. Your commit message shows *why*.

---

## Good vs Bad

### Good

```
feat(store): add OAuth2 integration

Implement Google sign-in. Users can now authenticate
using their Google account instead of email/password.

Closes #123
```

### Bad

```
fixed stuff
WIP
asdfgh
Updated file.css
```

---

## Quick Reference

```
feat|fix|refactor|perf|build|chore(scope): description

Rules:
- Subject: max 50 chars (hard limit: 72)
- Imperative ("Add" not "Added")
- Capitalize first letter, no period
- Body: wrap at 72, explain WHY not HOW
```

---

## Sources

- [How to Write a Git Commit Message](https://cbea.ms/git-commit/) - Chris Beams
- [A Note About Git Commit Messages](https://tbaggery.com/2008/04/19/a-note-about-git-commit-messages.html) - Tim Pope
- [Conventional Commits v1.0.0](https://www.conventionalcommits.org/en/v1.0.0/)