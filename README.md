# Skwd-wall

> [!CAUTION]
> Skwd-wall went through a complete backend rewrite to Rust as of 19/04/2026. Things might have stopped working - Please report them using the issue tracker on GitHub. I am just one person and it is easy for me to miss things when testing three different OS:es with different setups.
>
> Using the pre-rewrite Skwd-wall? You will have to reinstall Skwd-wall to get updates as breaking changes has happened to support the Rust backend. This also includes your keybind configurations. On the bright side installation is super easy now :)

![Stars](https://img.shields.io/github/stars/liixini/skwd-wall?style=for-the-badge)
![License](https://img.shields.io/github/license/liixini/skwd-wall?style=for-the-badge)
![Last Commit](https://img.shields.io/github/last-commit/liixini/skwd-wall?style=for-the-badge)
![Repo Size](https://img.shields.io/github/repo-size/liixini/skwd-wall?style=for-the-badge)
![Issues](https://img.shields.io/github/issues/liixini/skwd-wall?style=for-the-badge)

![Arch Linux](https://img.shields.io/badge/Arch_Linux-1793D1?style=for-the-badge&logo=archlinux&logoColor=white)
![Fedora](https://img.shields.io/badge/Fedora-51A2DA?style=for-the-badge&logo=fedora&logoColor=white)
![NixOS](https://img.shields.io/badge/NixOS-5277C3?style=for-the-badge&logo=nixos&logoColor=white)

<img alt="image" src="https://github.com/user-attachments/assets/157100e4-88e5-4542-8eba-fea0576e8801" />
<img alt="image" src="https://github.com/user-attachments/assets/367a6a0d-a384-490d-abe2-98c053ff9ffc" />
<img alt="image" src="https://github.com/user-attachments/assets/b73eff46-fa62-40cd-9109-9170adaa1dc5" />
<img alt="image" src="https://github.com/user-attachments/assets/577611da-d03d-4bf7-88c3-69b782cac668" />
<img alt="image" src="https://github.com/user-attachments/assets/a221355a-2530-42bb-a9c1-54d31062c7af" />
<img alt="image" src="https://github.com/user-attachments/assets/533f01d9-598b-4f02-881d-86cda7f28749" />


### A video is a thousand pictures - Sun Tzu (probably)

https://github.com/user-attachments/assets/c03ae4c8-76ea-42d0-8557-5db2465e6b2c

## What is Skwd-wall?

An image/video/Wallpaper Engine wallpaper selector from my shell [Skwd](https://www.github.com/liixini/skwd) with maximalist animations and more flair than you can shake a stick at. Now separated as a standalone component for use with other shells.

## What's cool about it?
- **Unified media support**: Handle images, videos, and even Wallpaper Engine scenes in one place.
- **Colour sorting**: All your images, videos and WE scenes are automatically sorted by hue and saturation into one of 13 colour groups.
- **Matugen colour schemes**: Automatically extracts colour palettes from wallpapers for a cohesive UI - this includes video & WE. Have an external Matugen configuration already? No problem - simply point to it in the Matugen configuration tab.
- **Execute refresh scripts**: Many applications need a script to refresh its theming - why? I don't know, but they do. You can set each Matugen target to also execute a script at the end of the pipeline should the program you're theming require it.
- **Postprocessing**: Need to do fancier stuff? Maybe you want to call an external program with the wallpaper you just applied? Skwd-wall has you covered. It supports sending commands after selecting a wallpaper with useful data placeholders like %path%, %type% and %name%.
- **Configurable**: Most dimensions and options are configurable to fit your preferences.
- **Tag system**: Support for any tag you want for easy and quick search and filtering, but also Ollama integration for automated tagging.
- **Restores wallpaper on boot**: It tracks the last wallpaper application command and reruns it on next boot.
- **So many filter options**: Filter by type, colour, recently added, tags, favourites, and more.
- **Wallhaven.cc & Steam Wallpaper Engine Workshop integration**: Browse and set wallpapers directly from wallhaven.cc or Steam and apply directly to your desktop with the click of a button.
- **Three different visual presentation styles**: A parallelogram slice carousel style, a more traditional grid style and a hexagon style, all with lots of animations and options of course!
- **Built-in image optimization**: Skwd-wall can automatically convert all images to webp as well as downscale the resolution to match your maximum resolution. The system is completely optional but useful when you are asking yourself why you have 70 GB of wallpapers.
- **Built-in video optimization** *(WIP)*: Video conversion to hevc with bitrate and resolution control is coming soon.
- **Retention out of the box**: Accidentally converted your 4k wallpaper to 1080p webp? No problem - Skwd-wall moves the originals to a retention directory and only deletes them automatically after the retention period on opt-in.
- **Wide system support**: Anywhere you can resolve the dependencies below and you have a wlr-layer-shell capable compositor, this should run.
- **For those that don't speak nerd**: That means it works on OS:es like Arch, Fedora & NixOS and downstream OS:es like CachyOS and Nobara but also with things like KDE Plasma, Hyprland, Sway or Niri - pretty much any Wayland compositor. It does **not** work with GNOME.
- **Keybinds**: A lot of features in Skwd-wall is navigatable by keybinds, available for reference under the keybind configuration tab.
- **Random wallpaper**: Press once for a random wallpaper, keep toggled for a random wallpaper every X seconds, X being configurable in the settings.
- **Different wallpapers on different monitors**: *(WIP)* You can enable an option to have a popup that allows you to select which monitors your wallpaper should apply to. Image only currently, but working on support for video and WE.

## The long story - Personal motivation and development practices
This is part of my personal shell Skwd that I have broken out into standalone components because it was a popular request.
I develop it because I feel most wallpaper selectors are very boring traditional grids, lack filtering options that don't accomodate people like me who have thousands of wallpapers and also because it is fun!

Note that **I use AI tooling** in my development just like I do in my professional life, however most of the code is mine including the stupid decisions.

## Performance
Performance is a big consideration for Skwd.

The daemon takes a tiny 10 MB of RAM and is the only permanent thing taking memory on your system.
As Skwd-wall shuts down entirely between uses it has zero footprint when not in use, and while in use it takes between 150 to 300 MB of RAM depending on the size of your wallpaper collection.

As Skwd-wall isn't simply flipping between hidden and shown fast startup times is a must and it takes about 0.2 seconds to start, with an optional 400 ms fade in animation.

## Dependencies
<Details>
<Summary>Dependency list</Summary>

### Required

| Dependency                                                                                                                                                                                 | Why                                                                                                                  |
|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------|
| [quickshell](https://github.com/quickshell-mirror/quickshell)                                                                                                                              | It is written with Quickshell... so um yeah                                                                          |
| [Qt6 Multimedia](https://doc.qt.io/qt-6/qtmultimedia-index.html)                                                                                                                           | Powers the video previews                                                                                            |
| [awww](https://codeberg.org/LGFae/awww)                                                                                                                                                    | Wallpaper software for images with cool effects when applying the wallpaper                                          |
| [matugen](https://github.com/InioX/matugen)                                                                                                                                                | Automatic colour extraction from the wallpapers                                                                      |
| [ffmpeg](https://ffmpeg.org)                                                                                                                                                               | Used to generate thumbnails from videos to have something to run Matugen on                                          |
| [ImageMagick](https://imagemagick.org)                                                                                                                                                     | Gives us the dominant colour and saturation for colour sorting                                                       |
| [curl](https://curl.se)                                                                                                                                                                    | Qt has a built in web request function but curl just works better                                                    |
| [sqlite3](https://sqlite.org)                                                                                                                                                              | We cache all our data in the database for lookups. JSON doesn't really like when you have 8 MB worth of data in a JSON file |
| [inotify-tools](https://github.com/inotify-tools/inotify-tools)                                                                                                                            | Used to see if there's changes in the wallpaper directories to trigger add or delete functionality                   |
| [Nerd Fonts Symbols](https://www.nerdfonts.com)                                                                                                                                            | UI icons, as they're symbols we can colour them any way we like which is good when Matugen does the colouring        |
| [Roboto](https://fonts.google.com/specimen/Roboto) + [Roboto Condensed](https://fonts.google.com/specimen/Roboto+Condensed) + [Roboto Mono](https://fonts.google.com/specimen/Roboto+Mono) | The main fonts used in Skwd                                                                                          | | And this too                                                                                                         |
| [mpvpaper](https://github.com/GhostNaN/mpvpaper)                                                                                                                                           | Video wallpaper backend                                                                                              |
| [jq](https://jqlang.github.io/jq/)                                                                                                                                                         | JSON processing for various internal operations                                                                      |
| [Material Design Icons](https://pictogrammers.com/library/mdi/)                                                                                                                            | Not all symbols are in nerd fonts symbols, so this supplements that                                                  |

### Optional

| Dependency                                                               | Why                                                                                                                                                                                                                                    |
|--------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [ollama](https://ollama.com)                                             | Used for computer vision to automatically tag wallpapers. Disabled by default - enable in settings                                                                                                                                     |
| [steamcmd](https://developer.valvesoftware.com/wiki/SteamCMD)            | Steam Workshop integration for the in-app browsing of Wallpaper Engine wallpapers. Requires API keys and an actual purchased copy of Wallpaper Engine. Disabled by default but the functionality is in there if you want to try it out |
| [linux-wallpaperengine](https://github.com/Almamu/linux-wallpaperengine) | Wallpaper Engine scene rendering. **_Not required if you only want video wallpapers_**!                                                                                                                                                | |

</Details>

## Install

### Base wallpaper path
The base wallpaper path is ~/Pictures/Wallpapers so that's where you put your pictures and videos unless you want to customise and put them elsewhere.

### Compositor-specific examples on how to launch

```
# Niri
Mod+T hotkey-overlay-title="Skwd-wall" { spawn "skwd wall toggle"; }

# Hyprland
bind = SUPER+T, exec, skwd wall toggle

# KDE Plasma - Use the shortcut app
skwd wall toggle
```

Research how to do this in your specific compositor, I'm sure it supports keybinds.

### Arch Linux

<Details>
<Summary>Arch Linux, CachyOS, EndevourOS, Manjaro, Garuda Linux etc.</Summary>

```sh
# Install Skwd-wall and all its dependencies
yay -S skwd-wall

# Enable Skwd-daemon:
systemctl --user enable --now skwd-daemon.service

# Launch Skwd-wall. Bind this command to a key in your compositor for quick access:
skwd wall toggle
```

If you're updating Skwd-wall, note that Skwd-wall is two applications - Skwd-wall and Skwd-daemon.
Skwd-daemon is automatically installed as part of installing Skwd-wall, but if you're updating and not updating all packages you need to
either use `yay -S skwd-wall --devel` or `yay -S skwd-wall skwd-daemon` 

> **Note:** `yay` is an AUR helper. If you don't have it, install it or use another helper like `paru`.

</Details>

### NixOS

<Details>
<Summary>NixOS</Summary>

Add the flake input to your `flake.nix`:

```nix
{
  inputs = {
    skwd-wall.url = "github:liixini/skwd-wall";
  };
}
```

Then add the package to your `configuration.nix`:

```nix
{ pkgs, inputs, ... }:
{
  environment.systemPackages = [
    inputs.skwd-wall.packages.${pkgs.stdenv.hostPlatform.system}.default
  ];
}
```

```sh
# Rebuild:
sudo nixos-rebuild switch

# Enable Skwd-daemon:
systemctl --user enable --now skwd-daemon.service

# Launch Skwd-wall. Bind this command to a key in your compositor for quick access:
skwd wall toggle
```

</Details>

### Fedora

<Details>
<Summary>Fedora, Bazzite, Nobara etc.</Summary>

```sh
# Enable the COPR repos
sudo dnf copr enable errornointernet/quickshell
sudo dnf copr enable scottames/awww
sudo dnf copr enable piixini/skwd

# Install skwd-wall:
sudo dnf install skwd-wall

# Enable Skwd-daemon:
systemctl --user enable --now skwd-daemon.service

# Launch Skwd-wall. Bind this command to a key in your compositor for quick access:
skwd wall toggle
```

</Details>

## Compositor-specific tweaks (KDE Plasma, Hyprland etc)

### Hyprland
<Details>
<Summary>Hyprland fixes and tweaks</Summary>
In testing I experienced issues with NixOS + systemctl service autostart on Hyprland.

This was resolved by adding a basic exec once to `hyprland.conf`, e.g.
  
`exec-once = systemctl --user start skwd-daemon`

I am sure there's a much more graceful way to solve this, but I am not a Hyprland user and this works.
</Details>

### KDE Plasma
<Details>
<Summary>KDE Plasma fixes and tweaks</Summary>
Skwd-wall auto-detects KDE Plasma and uses native Plasma APIs instead of awww/mpvpaper.

**Static wallpapers** work out of the box via `plasma-apply-wallpaperimage` - you don't have to do anything, it just works but still good to know.

**Video wallpapers** require the [Smart Video Wallpaper Reborn](https://github.com/luisbocanegra/plasma-smart-video-wallpaper-reborn) Plasma plugin. Without it, video wallpapers will not work on KDE.

### Installing the video wallpaper plugin

**KDE Store (any distro):**

Install via the KDE Store: right click Desktop > Desktop and Wallpaper > Get New Plugins > search "Smart Video Wallpaper Reborn" (or just select it, should be in the top)

After installing, Skwd-wall will automatically use the plugin for video wallpapers. No configuration required.

**Arch Linux:**
```sh
yay -S plasma6-wallpapers-smart-video-wallpaper-reborn
```

**Fedora:**
```sh
sudo dnf install plasma-smart-video-wallpaper-reborn
```
</Details>

## Optional - Wallpaper Engine, Steamcmd & Ollama
Skwd-wall supports two optional features - Wallpaper Engine wallpapers through [Linux Wallpaper Engine](https://github.com/Almamu/linux-wallpaperengine) and automated tagging for the tag search feature using computer vision through [Ollama](https://ollama.com/).


### Wallpaper Engine
As far as I am aware to use Wallpaper Engine on Linux you have to own the Steam application.
Swkd-wall finds Wallpaper Engine wallpapers automatically and sorts them based on type (video or Wallpaper Engine Scene). You can use the Steam application to manage your Steam Engine wallpaper collection.

However if you don't want to use the default Wallpaper Engine browser you can use Skwd-wall's internal one, which uses [Steamcmd](https://developer.valvesoftware.com/wiki/SteamCMD) which is Valve's Command Line Interface for Steam to search the Workshop behind the scenes.

You won't have to interact with Steamcmd more than logging in once so that Skwd-wall can use your logged in Steamcmd to browse the Workshop and download Wallpaper Engine workshop items (wallpapers) for you and Skwd-wall will warn you if your token has expired or needs refreshing (read: you need to log into Steam again).

Skwd-wall **does not** handle any of your Steam credentials - this is all done through Valve's Steamcmd - it simply tries to use Steamcmd and either you're logged in or you're not. This means that I will not be implementing in-app login flows for this - I do not wish to handle any authentication and I leave this solely on the shoulders of Valve.

### Ollama
Ollama is a local-only LLM that in Skwd-wall's case is used to automatically tag wallpapers as it is a very easy way to setup computer vision.
You simply need to enter the Ollama URL, download a model that supports computer vision e.g. `ollama pull gemma3:4b` and select the model in the field in Skwd-wall's Ollama settings - it automatically fetches installed models from Ollama for you.

You then press the O-button in the filter bar that is start / stop and after a couple of wallpapers tagged it will give you an estimated time until completion. You are safe to close Skwd-wall at this point as Skwd-daemon is the one executing the job and listening to the start/stop commands from Skwd-wall.

This does not overwrite existing tags should you already have tags set up. In testing I've found gemma3:4b to be very good at tagging while also being reasonable on the hardware requirements.

There's also a WIP tag consolidation system where similar tags get merged, but it is highly experimental right now.

## Acknowledgements
Ilyamiro1 for the 250 IQ idea to use duckduckgo to retrieve wallpapers which made me realise wallhaven.cc & Steam have API:s for similar functionality.
Also for implementing my ideas of parallelogram animations and colour sorting in his wallpaper selector - just happy people like my whacky ideas.

Horizon0427 for his [excellent hexagon wallpaper selector](https://github.com/Horizon0427/Arch-Config) from which I designed my hexagon style presentation entirely, with added animations and other features.

Happyzxzxz for showing me the Nix wizard way to do NixOS things.

## License

[MIT](LICENSE)


<!-- Fix #47 -->
