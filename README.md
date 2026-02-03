# Learning Bevy

## Milesones 1

Initialize a basic bevy project. Print a debug message once per second

- Initial repo created
- Nix flake enabled
- Native and web builds working.

#### Commands

```
bevy run
```

```
bevy run web --open
```

## Milestones 2

Render a few rectangles into the scene.

- Added a `WorldRoot` parent with a rebuild flow that despawns and recreates all child entities on hot patch.
- Integrated `bevy_hotpatching_experiments` and Dioxus hot-patch tooling

### Commands

```
dx serve --hot-patch --debug-symbols
```
