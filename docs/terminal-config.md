# How To Enable <kbd>Option/Alt</kbd>+<kbd>\<number\></kbd> Shortcut

It might be disabled by default in your terminal, here is how to make it work in some apps.

## VS Code integrated terminal

In your VS Code settings (`settings.json`), add:

```json
"terminal.integrated.macOptionIsMeta": true
```

## Ghostty

Add the following to your Ghostty config file (`~/.config/ghostty/config`):

```
macos-option-as-alt = true
```

> **Note:** A full app restart (not just a new window) may be required for the change to take effect.

## Zed integrated terminal

In your Zed settings (`~/.config/zed/settings.json`), add:

```json
{
  "terminal": {
    "option_as_meta": true
  }
}
```

## iTerm 2

In Preferences go to `Profiles`, select your profile and go to `Keys`. At the bottom set `Left Option (⌥) Key` to `Esc+`.

![iTerm 2 app preferences window](../img/iTerm-Option-key@2x.png)

## macOS Terminal

In Preferences go to `Profiles`, select your profile and go to `Keyboard`. Enable `Use Option as Meta key` checkbox.

![macOS Terminal app preferences window](../img/Terminal-Option-key@2x.png)
