# Skoll — ᛊᚲᛟᛚᛚ

Lightning-fast, minimalist Wayland application launcher and calculator built in Rust. Designed for a distraction-free "zen" workflow.

## About the Name

In Norse mythology, **Skoll** is the wolf that chases the sun across the sky, swallowing it at Ragnarök. Skoll is a child of Fenrir (which give the name to my previous TUI launcher).

## UI preview

TODO <!-- ![Skoll Preview](screenshots/skoll_preview_1.gif) -->

## About the project

TODO

## Screenshots

TODO

## Configuration

You can configure colors with a json file in ```~/.config/skoll/config.json```:

```js
{
    "background_color": [red,green,blue,alpha] | "random" | null,
    "accent_color": [red,green,blue,alpha] | "random" | null
}
```

For example:

```js
{
    "background_color": [0.1, 0.2, 0.2, 0.9],
    "accent_color": [0.6, 0.1, 0.0, 1.0]
}
```

- `background_color`: Accepts RGBA color values or `"random"`. When set to `"random"`, the color is automatically generated (Note: it is really random, at the moment, not a "nice random"). This is the color for the layer covering the entire screen when the launcher is visible.

- `accent_color`: Accepts RGBA color values or `"random"`. When set to `"random"`, the color is automatically generated (Note: it is really random, at the moment, not a "nice random"). This is the color of the selector of existing windows.

Default values are the following:

```js
{
    "background_color": [0.0, 0.0, 0.0, 0.85],
    "accent_color": [1.0, 0.2, 0.5, 1.0]
}
```

## TODOs

- ~~Cache app icons~~ Done!
- Make Niri integration optional
- ~~Make accent color configurable~~ Done!
- ~~Make background opacity / color configurable~~ Done!
- Make min refresh rate configurable
- Add history to the calculator
- Add capability of variable usage to the calculator
- Cache app icons _before_ first opening (at daemon launch, for example)