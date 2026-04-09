# Skwd-wall

![License](https://img.shields.io/github/license/liixini/skwd-wall?style=for-the-badge)
![Last Commit](https://img.shields.io/github/last-commit/liixini/skwd-wall?style=for-the-badge)
![Repo Size](https://img.shields.io/github/repo-size/liixini/skwd-wall?style=for-the-badge)
![Issues](https://img.shields.io/github/issues/liixini/skwd-wall?style=for-the-badge)

![Arch Linux](https://img.shields.io/badge/Arch_Linux-1793D1?style=for-the-badge&logo=archlinux&logoColor=white)
![Fedora](https://img.shields.io/badge/Fedora-51A2DA?style=for-the-badge&logo=fedora&logoColor=white)
![NixOS](https://img.shields.io/badge/NixOS-5277C3?style=for-the-badge&logo=nixos&logoColor=white)

<img width="2560" height="1439" alt="image" src="https://github.com/user-attachments/assets/157100e4-88e5-4542-8eba-fea0576e8801" />
<img width="2562" height="1440" alt="image" src="https://github.com/user-attachments/assets/367a6a0d-a384-490d-abe2-98c053ff9ffc" />
<img width="2558" height="1439" alt="image" src="https://github.com/user-attachments/assets/b73eff46-fa62-40cd-9109-9170adaa1dc5" />
<img width="2560" height="1440" alt="image" src="https://github.com/user-attachments/assets/577611da-d03d-4bf7-88c3-69b782cac668" />
<img width="2560" height="1439" alt="image" src="https://github.com/user-attachments/assets/a221355a-2530-42bb-a9c1-54d31062c7af" />

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

## What isn't cool about it yet?
- **Subdirectories**: Currently working on subdirectory support.
- **COPR/AUR/Nix Flake**: Looking into creating packages for easy installation and updates.
- **Keybind customization**: Investigating being able to customise keybinds freely to suit your preferences.

## The long story - Personal motivation and development practices
This is part of my personal shell Skwd that I have broken out into standalone components because it was a popular request.
I develop it because I feel most wallpaper selectors are very boring traditional grids, lack filtering options that don't accomodate people like me who have thousands of wallpapers and also because it is fun!

Note that **I use AI tooling** in my development just like I do in my professional life, however most of the code is mine including the stupid decisions.

## Performance
Performance has been a large consideration making this (a bit harder to flick through four thousand wallpapers smoothly in 10 seconds than one might think!), and the application is in two parts - the daemon and the GUI.
However this is not designed to be the leanest wallpaper selector out there and uses aggressive caching of data to support the smooth operations.

In testing the daemon consumes about 100 MB (PSS) and deals with things like background image & video optimisation (if enabled) and updating and caching the database with new wallpapers as they get added from the supported sources.

The GUI on the other hand sits at about 175 MB (PSS) for about a 1000 wallpapers, but scaling with roughly 60 KB / wall. This is a deliberate design decision to cache as much data as possible to enable smooth retrieval of objects with any arbitrary search and quick.

The wallpaper applications are spawned detached meaning you can completely kill the daemon with no ill effect outside of the cold start taking a bit longer as new wallpapers are processed on start by the daemon as it detects them.

## Dependencies
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
| [Material Design Icons](https://pictogrammers.com/library/mdi/)                                                                                                                            | Not all symbols are in nerd fonts symbols, so this supplements that                                                  |

### Optional

| Dependency                                                               | Why                                                                                                                                                                                                                                    |
|--------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [mpvpaper](https://github.com/GhostNaN/mpvpaper)                         | Required for video wallpapers                                                                                                                                                                                                          |
| [ollama](https://ollama.com)                                             | Used for computer vision to automatically tag wallpapers. Disabled by default - enable in settings                                                                                                                                     |
| [steamcmd](https://developer.valvesoftware.com/wiki/SteamCMD)            | Steam Workshop integration for the in-app browsing of Wallpaper Engine wallpapers. Requires API keys and an actual purchased copy of Wallpaper Engine. Disabled by default but the functionality is in there if you want to try it out |
| [linux-wallpaperengine](https://github.com/Almamu/linux-wallpaperengine) | Wallpaper Engine scene rendering. **_Not required if you only want video wallpapers_**!                                                                                                                                                | |

## Install

### Base wallpaper path
The base wallpaper path is ~/Pictures/Wallpapers so that's where you put your pictures and videos unless you want to customise and put them elsewhere.

### Before proceeding, read carefully:
Skwd-wall launches on demand and exits when you close the selector, keeping zero memory footprint when not in use but uses a daemon for doing background tasks like keeping long running jobs open. All cached data (thumbnails, color palettes, tags) is persisted to disk and reloaded on next launch.

Skwd-wall detatches all wallpaper processes meaning you can kill it entirely after setting a wallpaper if needed. I suggest `pkill -f "skwd-wall"`.

### To run the software

Start the daemon:
```
quickshell -p /path/to/installation/daemon.qml
```

Then bind a key to toggle the selector via IPC:

```
# Niri
Mod+T hotkey-overlay-title="Skwd-wall" { spawn "quickshell ipc -p ~/skwd-wall/daemon.qml call wallpaper toggle"; }

# Hyprland
bind = SUPER+T, exec, quickshell ipc -p ~/skwd-wall/daemon.qml call wallpaper toggle

# KDE Plasma - Use the shortcut app
quickshell ipc -p ~/skwd-wall/daemon.qml call wallpaper toggle
```

Research how to do this in your specific compositor, I'm sure it supports keybinds and executing on launch.

## KDE Plasma - If you're a KDE user start here before proceeding

Skwd-wall auto-detects KDE Plasma and uses native Plasma APIs instead of awww/mpvpaper (as KDE Plasma doesn't play nice like that).

**Static wallpapers** work out of the box via `plasma-apply-wallpaperimage` - no extra setup needed.

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

## Arch Linux
```
sudo pacman -S curl jq sqlite ffmpeg imagemagick inotify-tools ttf-nerd-fonts-symbols qt6-multimedia ttf-roboto ttf-roboto-mono
yay -S quickshell-git awww-bin matugen-bin ttf-material-design-icons-desktop-git
```

Optional: `sudo pacman -S ollama && yay -S mpvpaper steamcmd linux-wallpaperengine-git`

`git clone https://github.com/liixini/skwd-wall && cd skwd-wall`

# the -p part is for PATH, extend to match the path where you find daemon.qml
# set this up with your exec once of choice, such as a .desktop file, in your compositor etc.
quickshell -p daemon.qml

# this is the part you keybind somehow which launches the UI!
quickshell ipc -p daemon.qml call wallpaper toggle

Note that yay is an AUR (Arch User Repository) helper, so if you don't have that you will need to install it or alternatively another helper you prefer.

## NixOS

**Warning**! I am not a NixOS user. This is the trial and error configuration I used in my NixOS VM for testing.
If you are a NixOS user please make a pull request if you feel there's easier ways to do this because I am sure there are.

<details>
  <summary>Install instructions by me, trainee NixOS wizard</summary>
  Add the flake inputs for quickshell and awww to your `flake.nix`:

```nix
{
  inputs = {
    quickshell.url = "github:quickshell-mirror/quickshell";
    awww.url = "git+https://codeberg.org/LGFae/awww";
  };
}
```

Pass inputs to your modules via `specialArgs`, then in `configuration.nix`:

```nix
{ pkgs, inputs, ... }:
{
  environment.systemPackages = with pkgs; [
    # Quickshell with QtMultimedia seemingly we need to use their pinned nixpkgs
    # to avoid Qt version mismatches between your system nixpkgs and the flake's
    (let
      qsPkgs = inputs.quickshell.inputs.nixpkgs.legacyPackages.${pkgs.stdenv.hostPlatform.system};
    in inputs.quickshell.packages.${pkgs.stdenv.hostPlatform.system}.default.withModules [
      qsPkgs.qt6.qtmultimedia
    ])
    inputs.awww.packages.${pkgs.stdenv.hostPlatform.system}.awww
    curl
    jq
    file
    sqlite
    ffmpeg
    imagemagick
    inotify-tools
    nerd-fonts.symbols-only
    roboto
    roboto-mono
    material-design-icons
    matugen
  ];
}
```

> **Note:** QtMultimedia is required for video previews. On KDE Plasma sessions it may work
> without the `.withModules` line since Plasma provides QtMultimedia system-wide, but on
> standalone compositors like Hyprland or Niri you **must** pass it explicitly via `.withModules`
> or the UI will silently fail to load.

Clone and run:

```sh
git clone https://github.com/liixini/skwd-wall && cd skwd-wall

# the -p part is for PATH, extend to match the path where you find daemon.qml
# set this up with your exec once of choice, such as a .desktop file, in your compositor etc.
quickshell -p daemon.qml

# this is the part you keybind somehow which launches the UI!
quickshell ipc -p daemon.qml call wallpaper toggle
```
</details>

<details>
  <summary>Currently reported as having issues - proceed at your own risk! Install instructions using flakes by happyzxzxz</summary>

  1. Ensure Flakes are enabled in your `configuration.nix`:
  `nix.settings.experimental-features = [ "nix-command" "flakes" ]`
  2. Also add this in `configuration.nix` (Sorry, I couldn't figure out how to wrap it all in the flake)

```
# Quickshell with QtMultimedia seemingly we need to use their pinned nixpkgs
# to avoid Qt version mismatches between your system nixpkgs and the flake's
environment.systemPackages = with pkgs; [
  (let
    qsPkgs = inputs.quickshell.inputs.nixpkgs.legacyPackages.${pkgs.stdenv.hostPlatform.system};
  in inputs.quickshell.packages.${pkgs.stdenv.hostPlatform.system}.default.withModules [
    qsPkgs.qt6.qtmultimedia
  ])
];
```
  3. And this in your `/etc/nixos/flake.nix`:
  ```
  {
    inputs = {
      quickshell.url = "github:quickshell-mirror/quickshell";
    };
  }
```
  4. Rebuild your system: `nixos-rebuild switch`

Next you can run `nix profile install .` in the repo folder to install it on your system.
Once installed you can launch daemon with `skwd-wall-daemon` and toggle with `skwd-wall-toggle`

</details>

Optional: add `ollama`, `mpvpaper` to your system packages as needed.

### Fedora

Enable the COPR repos for quickshell and awww:

```sh
sudo dnf copr enable errornointernet/quickshell
sudo dnf copr enable scottames/awww
```

Install dependencies:

```sh
sudo dnf install quickshell awww jq curl sqlite ffmpeg ImageMagick inotify-tools \
  qt6-qtmultimedia google-roboto-fonts google-roboto-condensed-fonts google-roboto-mono-fonts
  
  Optional: `sudo dnf install ollama mpvpaper`
```

Install matugen via cargo:

```sh
cargo install matugen
```

If cargo isn't already in your PATH, add it to bash or zsh or whatever you use, bash example below:
```
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

Install Nerd Fonts Symbols and Material Design Icons:

```sh
mkdir -p ~/.local/share/fonts
curl -fLo ~/.local/share/fonts/SymbolsNerdFontMono-Regular.ttf \
  https://github.com/ryanoasis/nerd-fonts/raw/HEAD/patched-fonts/NerdFontsSymbolsOnly/SymbolsNerdFontMono-Regular.ttf
curl -fLo ~/.local/share/fonts/MaterialDesignIconsDesktop.ttf \
  https://github.com/Templarian/MaterialDesign-Desktop-Font/raw/HEAD/MaterialDesignIconsDesktop.ttf
fc-cache -fv
```

Clone and run:

```sh
git clone https://github.com/liixini/skwd-wall && cd skwd-wall

# the -p part is for PATH, extend to match the path where you find daemon.qml
# set this up with your exec once of choice, such as a .desktop file, in your compositor etc.
quickshell -p daemon.qml

# this is the part you keybind somehow which launches the UI!
quickshell ipc -p daemon.qml call wallpaper toggle
```



## Acknowledgements
Ilyamiro1 for the 250 IQ idea to use duckduckgo to retrieve wallpapers which made me realise wallhaven.cc & Steam have API:s for similar functionality.
Also for implementing my ideas of parallelogram animations and colour sorting in his wallpaper selector - just happy people like my whacky ideas.

Horizon0427 for his [excellent hexagon wallpaper selector](https://github.com/Horizon0427/Arch-Config) from which I designed my hexagon style presentation entirely, with added animations and other features.

## License

[MIT](LICENSE)
