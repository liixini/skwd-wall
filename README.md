<img width="2800" height="1640" alt="Skwd wall cover" src="https://github.com/user-attachments/assets/2d95a1ae-ab01-433a-a21a-57d07a2ccc76" />

> [!IMPORTANT]
> Skwd-wall v2 beta is here!
> Join us on [Discord](https://discord.gg/cgxy8EEVmz) if you want to be part of the steering the direction of the beta, report bugs or just chat :)
>
> Skwd-wall v2 is the most ambitious software I've ever written, and as such I am expecting at least one bug, maybe two. Please don't hesitate to create a GitHub issue for any issues you encounter or suggestions you may have.
>
> Current known issues / WIP:
>
> WIP: Wallpaper Engine inconsistencies / bugs in comparison to (the real) Wallpaper Engine. This part of the application is still Work in Progress and does not have perfect coverage yet. Please report any issues you may find with Wallpaper Engine scenes ❤️
>
> WIP: Debian version
>
> WIP: A wiki & better documentation. Skwd-wall is packed with tons of features, but almost nothing is explained in detail in writing.

![Stars](https://img.shields.io/github/stars/liixini/skwd-wall?style=for-the-badge)
![License](https://img.shields.io/github/license/liixini/skwd-wall?style=for-the-badge)
![Last Commit](https://img.shields.io/github/last-commit/liixini/skwd-wall?style=for-the-badge)
![Repo Size](https://img.shields.io/github/repo-size/liixini/skwd-wall?style=for-the-badge)
![Issues](https://img.shields.io/github/issues/liixini/skwd-wall?style=for-the-badge)

![Arch Linux](https://img.shields.io/badge/Arch_Linux-1793D1?style=for-the-badge&logo=archlinux&logoColor=white)
![CachyOS](https://img.shields.io/badge/CachyOS-00AA88?style=for-the-badge&logo=cachyos&logoColor=white)
![Fedora](https://img.shields.io/badge/Fedora-51A2DA?style=for-the-badge&logo=fedora&logoColor=white)
![Bazzite](https://img.shields.io/badge/Bazzite-6554C0?style=for-the-badge)
![NixOS](https://img.shields.io/badge/NixOS-5277C3?style=for-the-badge&logo=nixos&logoColor=white)
![Debian](https://img.shields.io/badge/Debian-A81D33?style=for-the-badge&logo=debian&logoColor=white)
![Ubuntu](https://img.shields.io/badge/Ubuntu-E95420?style=for-the-badge&logo=ubuntu&logoColor=white)
![Pop!_OS](https://img.shields.io/badge/Pop!__OS-48B9C7?style=for-the-badge&logo=popos&logoColor=white)
![Linux Mint](https://img.shields.io/badge/Linux_Mint-86BE43?style=for-the-badge&logo=linuxmint&logoColor=white)

![Niri](https://img.shields.io/badge/Niri-4B83CD?style=for-the-badge&logo=niri&logoColor=white)
![Hyprland](https://img.shields.io/badge/Hyprland-58E1FF?style=for-the-badge&logo=hyprland&logoColor=black)
![KDE Plasma](https://img.shields.io/badge/KDE_Plasma-1D99F3?style=for-the-badge&logo=kdeplasma&logoColor=white)
![Mango](https://img.shields.io/badge/Mango-FF9900?style=for-the-badge&logo=wayland&logoColor=white)
![Sway](https://img.shields.io/badge/Sway-68751C?style=for-the-badge&logo=sway&logoColor=white)
![COSMIC](https://img.shields.io/badge/COSMIC-FFAD00?style=for-the-badge&logo=system76&logoColor=black)
![labwc](https://img.shields.io/badge/labwc-555555?style=for-the-badge&logo=wayland&logoColor=white)

### A video is a thousand pictures - Sun Tzu (probably)



https://github.com/user-attachments/assets/46efe308-806d-466d-8834-3052134f00eb



## What is Skwd-wall?
> Something so beautiful and aesthetic yet so lightweight to run amazingly on a celeron
> 
> You have returned the hope I lost
>
> \- FutureKismo

> i honestly loved v1. such an amazing piece of kit, and my machine didn't mind running it. but v2 is like going from "family sedan fast" to "ferrari fast"
>
> \- WanderingAstarael

Skwd-wall is a wallpaper manager for most Linux distros and WM:s that can show Wallpaper Engine scenes (and browse them too!), images and videos and does so much more performant than most wallpaper daemons out there.

We got all the features from changing the layout to right-to-left for languages read that way to letting you put videos in the Niri backdrop (paused unless you're looking, of course), and so much more - if you can think of it for a wallpaper manager chances are high Skwd-wall does it.

So what about that performance claim? Don't take my word for it, I come with charts!

<img width="1500" height="2466" alt="performance" src="https://github.com/user-attachments/assets/34586f27-daa1-4ded-b7cf-20a10bae3874" />

### What does it do besides apply wallpapers? well...

<Details>
<Summary>What Skwd-wall v2 actually does</Summary>

- Displays images, videos and supported Wallpaper Engine scenes.
- Has four completely different GPU-rendered pickers: Slices, Geometric, Wall and Sandy. Because why not?
- Has out of the box colour theming for popular applications including KDE Plasma, but can easily be extended with Matugen scripts.
- Starts in about 150 ms, stops rendering when idle and exits completely when closed meaning no resources required unless you're switching wallpapers.
- Includes 39 (I counted them myself!) wallpaper transitions, from normal fades to turning every pixel into sand and firing it through a Möbius strip.
- Snapchat filters! ...I mean, quick Photoshop effects that you might want to apply and create new wallpapers from. You can stack several of them on top of each other too.
- Lets you describe the wallpaper you want using local visual search.
- Filters enormous libraries by type, Shape, resolution, folder, colour, tags, favourites, metadata and current weather (I'll let you decide how to use the last one).
- Browses Steam Workshop, Wallhaven, YouTube and Bing Daily from inside the picker.
- Applies different wallpapers, placement and audio to each display independently.
- Can lock displays against schedules and random changes.
- Builds manual or filtered playlists, with shuffle, sequential playback and separate display assignments.
- Runs absurdly specific schedules using time, date, sunrise, sunset, weather, battery, power and connected displays, combined with full conditional logic using ANY, ALL or OR and nestled groups.
- Pins wallpapers to individual Niri, Hyprland and KWin workspaces. Configuration only for now so WIP!
- Derives desktop colours from the wallpaper using Iris, Matugen, Wallust, Pywal, Caelestia, Noctalia, DMS or end-4... yeah um, I went a bit crazy.
- Includes a theme designer with all 50 Material colour roles, separate dark and light variants, saved palettes and wallpaper profiles. Saved colours carry through to app templates. Wallpaper hover previews are also available.
- Renders Wallpaper Engine scenes through Vulkan and exposes their switches, sliders and other editable properties.
- Provides a per-display wallpaper audio mixer for videos and scenes, because wallpapers having audio is important to some people (or so they told me in the github issues) and has a decision engine so that only 1 audio from several identical sources plays at once no matter which you mute or raise the volume on.
- Shares Vulkan devices and video decoders across displays, pauses idle playback and applies automatic battery limits.
- Uses dramatically less combined RSS + VRAM in the measured comparison: 345 MiB versus mpvpaper's 1704 MiB for video, and 6 MiB versus a 545 MiB static competitor average excluding awww (which once again is absolutely amazing).
- Comes with skwd-helm for scripting, history, playlists, audio, event hooks and portable look packs, should clicking wallpapers eventually become too boring for you and you want to automatically change your wallpaper every time you open Overwatch.

</Details>

### What does it work on?
Skwd-wall has intentional wide support and targets all major distributions, Desktop Environments and Wayland compositors.
However it does not support GNOME or Linux Mint's Cinnamon (yet!).

## Who is Skwd-wall not for?
If you're someone that can reasonably name all the wallpapers you have, they're all images and you have one monitor chances are high Skwd-wall is not for you.

But if you're an aspiring wallpaper collector, like the idea of a program that tries to solve every wallpaper and theming problem you might ever have or you're just curious about Skwd-wall you have come to the right place!

## The long story short - Personal motivation and development practices
This is part of my personal shell Skwd that I have broken out into a standalone component because it was a popular request.

I develop Skwd-wall because I feel most wallpaper selectors are very boring traditional grids, lack filtering options that don't accomodate people like me who have thousands of wallpapers, because video wallpaper daemons were so-so to ??? on the performance side of things and most of all because it is fun!

Note that **I use AI tooling** in my development just like I do in my professional life, however most of the (non-test, ain't no way I'm writing the 1000+ unit, integration and e2e tests that Skwd-wall v2 has manually) code is mine including the stupid decisions.

### Base wallpaper path
The default is a `Wallpapers` folder inside your desktop's Pictures directory, including localized names such as `Imágenes`. Existing `~/Pictures/Wallpapers` libraries keep their location. You can choose another folder in settings; spaces and non-English names work, and settings open even before you add a wallpaper.

### Download filters

In **Settings > Sources**, expand a source and enable **Show Apply button** to change several download filters before sending one search request. Press **Apply**, or press Enter in the search field, to search with all your changes. Each source has its own choice. The button is hidden by default, so each filter change searches immediately.

Expand **Wallhaven** or **Steam Workshop** to choose the default filters for each new browser session. Changes made while browsing last until the browser closes.

### Compositor-specific examples on how to launch
Skwd-wall-v2 comes with a .desktop file so you can launch it through your launcher. But should you wish to keybind it, this is how you do that.

```
# Niri
Mod+T hotkey-overlay-title="Skwd-wall" { spawn "skwd-wall-v2"; }

# Hyprland
hl.bind("SUPER + W", hl.dsp.exec_cmd("skwd-wall-v2"))

# KDE Plasma - Use the shortcut app or launch through your launcher
skwd-wall-v2
```

Research how to do this in your specific compositor.

## Installation
### Arch Linux, CachyOS, EndevourOS, Manjaro, Garuda Linux etc.

<Details>
<Summary>Arch Linux, CachyOS, EndevourOS, Manjaro, Garuda Linux etc.</Summary>

```sh
# These are terminal commands!
# AUR - downloads prebuilt binaries so you don't have to compile a ton of Rust.
# If you want to compile a ton of rust, simply drop the -bin on the two packages
yay -S skwd-wall-v2-bin skwd-lens-bin

# Required only if you use KDE Plasma
yay -S skwd-paper-plasma

# Enable the wallpaper daemon
systemctl --user daemon-reload
systemctl --user enable --now skwd-walld.service

# Run using:
skwd-wall-v2
```

> **Note:** `yay` is an AUR helper. If you don't have it, install it or use another helper like `paru`.

</Details>

### NixOS

<Details>
<Summary>NixOS</Summary>

<Details>
<Summary>Imperative Installation</Summary>

```
# Enable flakes if you don't already use them
nix --extra-experimental-features 'nix-command flakes'

# Install
nix profile install github:liixini/skwd-wall/nix#default

# If you're using KDE Plasma you need the KDE Plasma plugin
nix profile install github:liixini/skwd-wall/nix#skwd-paper-plasma

# run with (or put in a keybind, convenient script... up to you)...
skwd-wall-v2
```

</Details>

<Details>
<Summary>Declarative Installation provided by HomieDerPrakti</Summary>

This assumes a basic layout that looks like this:
```
.
├── configuration.nix
└── flake.nix
```
If you don't have a `flake.nix` and don't understand what it does, consider researching about nix flakes first, or using the imperative installation method.

```nix
# Enable flakes if you don't already use them
## configuration.nix
nix.settings.experimental-features = [
  "nix-command"
  "flakes"
];

# Add the input to flake.nix
## flake.nix
{
  description = "Example flake";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    skwd-wall = {
      url = "github:liixini/skwd-wall/nix";
    };
  };
  outputs = inputs@{ nixpkgs, ... }:
  {
    nixosConfigurations = {
      # adjust to your actual hostname
      hostname = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        specialArgs = {
          inherit inputs;
        };
        modules = [ ./configuration.nix ];
      };
    };
  };
}

# Install
## configuration.nix
{ inputs, pkgs, ... }:
let
  skwd-wall = inputs.skwd-wall.programs.${pkgs.stdenv.hostPlatform.system}.default;
  skwd-paper-plasma = inputs.skwd-wall.programs.${pkgs.stdenv.hostPlatform.system}.skwd-paper-plasma;
in
{
  # import the NixOS Module if you're using it
  imports = [
    inputs.skwd-wall.nixosModules.default
  ];
  environment.systemPacakges = with pkgs; [
    skwd-wall
    # If you're using KDE Plasma, you also need the KDE Plasma plugin
    skwd-paper-plasma
  ];

  # If you use the module, you don't need to add the packages to environment.systemPacakges
  services.skwd-deck = {
    # enable the systemd service
    enable = true;
    extraPackages = [
      # If you're using KDE Plasma, you also need the KDE Plasma plugin
      skwd-paper-plasma
    ];
  };
}

# run with (or put in a keybind, convenient script... up to you)...
skwd-wall-v2
```

</Details>
</Details>
</Details>

### Fedora, Nobara etc.

<Details>
<Summary>Fedora, Nobara etc.</Summary>

```sh
# These are terminal commands!
# COPR
sudo dnf install dnf-plugins-core
sudo dnf copr enable piixini/skwd-wall-v2
sudo dnf install skwd-wall-v2 skwd-lens

# Required only if you use KDE Plasma
sudo dnf install skwd-paper-plasma

# Enable the wallpaper daemon
systemctl --user daemon-reload
systemctl --user enable --now skwd-walld.service

# Run using:
skwd-wall-v2
```

</Details>

### Bazzite KDE Plasma

<Details>
<Summary>Bazzite KDE Plasma</Summary>

```sh
# These are terminal commands!
# COPR
sudo dnf5 copr enable piixini/skwd-wall-v2 &&
sudo rpm-ostree install \
  skwd-wall-v2 skwd-deck skwd-paper \
  skwd-lens skwd-lens-model skwd-paper-plasma

# Reboot!
reboot

# Install the daemon that keeps wallpapers applied on reboot etc.
systemctl --user daemon-reload
systemctl --user enable --now skwd-walld.service

# Run using the below command, or keybind it or search for skwd-wall in your app launcher:
skwd-wall-v2
```

</Details>

### Debian, Ubuntu, Linux Mint etc - WIP

<Details>
<Summary>Debian-based</Summary>

Debian-based is currently WIP. Most is set up, but I haven't tested a full end-to-end installation.

</Details>

Open the wallpaper mixer directly from a launcher or keybinding:

```sh
skwd-wall-v2 --mixer
```

This starts Wall on the mixer screen, or opens the mixer in the running instance.

## Compositor-specific tweaks

### Niri overview wallpaper

In **Settings > Playback > Video**, set **Wallpaper layer** to **Background**, then add this to your Niri configuration:

```kdl
layer-rule {
    match namespace="^skwd-paper-backdrop$"
    place-within-backdrop true
}
```

Enable **Animate only in Niri overview** to pause the active wallpaper while the overview is closed. Manual, process, and fullscreen pause rules still apply when it opens. The separate `overviewBackdrop` option keeps its existing behaviour.

### KDE Plasma
<Details>
<Summary>KDE Plasma fixes and tweaks</Summary>

If you are using KDE Plasma you need the skwd-paper-plasma plugin.
Why do you ask? Because KDE Plasma controls the background layer, so we need to also be KDE Plasma hence the plugin.
You can find the plugin [here](https://github.com/liixini/skwd-paper-plasma) or you can install it through COPR or AUR.

</Details>

## Optional - Steamcmd or Steam Client
To retrieve wallpapers from Wallpaper Engine you have two options.

Skwd-wall can use [Steamcmd](https://developer.valvesoftware.com/wiki/SteamCMD) or we can use your already installed Steam Client and pretend we're Wallpaper Engine. E.g. if you want it to just work with the caveat of opening Steam - use option 2. If you want a robust option that works simply by opening Skwd-wall but requires you acquiring a Steam Web API key, use option 1. Mind you the Steam Client integration is WIP and may not work on all platforms (even though it should).

**API key + steamcmd (recommended):** add a Steam Web API key and set the backend to steamcmd. Browsing and downloads both work, no running Steam needed.

**Steam Client:** install the optional `skwd-deck-steamworks` package and keep native Steam running. The Fedora COPR install does not include this helper. Use SteamCMD if you have not installed the separate companion RPM.

You won't have to interact with Steamcmd more than logging in once so that Skwd-wall can use your logged in Steamcmd to browse the Workshop and download Wallpaper Engine workshop items (wallpapers) for you and Skwd-wall will warn you if your token has expired or needs refreshing (read: you need to log into Steam again).

Skwd-wall **does not** handle any of your Steam credentials - this is all done through Valve's Steamcmd - it simply tries to use Steamcmd and either you're logged in or you're not. This means that I will not be implementing in-app login flows for this - I do not wish to handle any authentication and I leave this solely on the shoulders of Valve.

## Semantic Search
Skwd-wall uses SigLIP2 8/16 as the default multilingual vision-language encoder. If all of that made you feel like you aren't reading English, essentially SigLIP2 tags pictures in many languages which allows us to retrieve the correct match when you search for "Outono fofo e aconchegante", "cute cozy autumn" or "Süßer gemütlicher Herbst".
However, SigLIP2 8/16 is a "middle of the road" model chosen for a low footprint, and Skwd-wall supports plugging in more powerful models should you have access to hardware that is suited for it.

Skwd-wall automatically saves indexes any encoder has generated per model, meaning you can use as many as you wish and Skwd-wall will automatically index new wallpapers since your last use of a model.

## Acknowledgements
Ilyamiro1 for the 250 IQ idea to use duckduckgo to retrieve wallpapers which made me realise wallhaven.cc & Steam have API:s for similar functionality.
Also for implementing my ideas of parallelogram animations and colour sorting in his wallpaper selector - just happy people like my whacky ideas.

Horizon0427 for his [excellent hexagon wallpaper selector](https://github.com/Horizon0427/Arch-Config) from which I designed my hexagon style presentation entirely, with added animations and other features.

Happyzxzxz for showing me the Nix wizard way to do NixOS things.

Harman1307 for [Iris](https://github.com/Harman1307/iris) which I have reimplemented in large parts and extended.

Achno for [Gowall](https://github.com/Achno/gowall) which I have reimplemented using Rust and similarly extended.

InioX for [Matugen](https://github.com/InioX/matugen) that powers a lot of the WIP bridges between Pywal et. al. that I'm building.

Kian Blakey for [Pibble](https://github.com/kianblakley/pibble) which the view mode Depth is heavily based on.
