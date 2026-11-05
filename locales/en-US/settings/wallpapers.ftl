settings-wallpapers-displays-desc = What each display shows, and its placement.
settings-wallpapers-play = How wallpapers play
settings-wallpapers-play-desc = Engines, the layer they draw on, and scenes.
settings-wallpapers-transitions-desc = How one wallpaper changes into the next.
settings-wallpapers-awww-desc = The transition awww plays for images.
settings-wallpapers-drawing = Drawing
settings-wallpapers-drawing-desc = Prepared wallpapers draw themselves in.
settings-wallpapers-sound = Sound
settings-wallpapers-sound-desc = Volume, and when wallpaper audio steps aside.
settings-wallpapers-pause-desc = Pause animated wallpapers when a rule is met.
settings-wallpapers-plasma = Plasma lock screen
settings-wallpapers-plasma-desc = Which wallpaper Plasma shows when locked.
settings-wallpapers-niri = Niri overview
settings-wallpapers-niri-desc = Shown on niri. A backdrop behind the overview.
settings-wallpapers-display-more = More
settings-wallpapers-lock = Lock { $name }
settings-wallpapers-lock-hint = Only update it through the multipicker
settings-wallpapers-layer-bottom = Bottom
settings-wallpapers-gpu-hint = Falls back to Automatic
settings-wallpapers-timeout-hint = For very large images
settings-wallpapers-animated = Animated wallpapers
settings-wallpapers-scenes = Scenes
settings-wallpapers-scenes-hint = FPS cap · scaling
settings-wallpapers-particles-hint = Scenes only
settings-wallpapers-preview-hint = Frame rate
settings-wallpapers-quality-short = Sand needs at least { $ms } ms.
settings-wallpapers-quality-hint = Full is uncapped, Low caps at 30 fps
settings-wallpapers-sharpen-hint = While quality reduces resolution
settings-wallpapers-monitors = Monitors
settings-wallpapers-primary-hint = Animates for Primary only
settings-wallpapers-primary-blank = first available
settings-wallpapers-primary-only-hint = The rest use all monitors
settings-wallpapers-wave = Wave
settings-wallpapers-wave-hint = Width × height
settings-wallpapers-position-hint = center, top-left or 0.5,0.5
settings-wallpapers-bezier-hint = x1,y1,x2,y2
settings-wallpapers-bezier-blank = awww default
settings-wallpapers-invert-hint = Flip the origin's Y axis
settings-wallpapers-colours-files = Colours and files
settings-wallpapers-preparing = Preparing
settings-wallpapers-paper-ink = Paper and ink
settings-wallpapers-prepare = Prepare
settings-wallpapers-prepare-all = All wallpapers
settings-wallpapers-scope-view = Current view · { $count }
settings-wallpapers-pack-ready = Processing files ready.
settings-wallpapers-pack-none = No processing files yet.
settings-wallpapers-pack-blank = None yet
settings-wallpapers-drawing-blocked = Drawing is unavailable. Choose processing files first.
settings-wallpapers-volume = Volume
settings-wallpapers-other-apps = Other apps
settings-wallpapers-pause-apps = Pauses while listed applications run.
settings-wallpapers-pause-none = No pause rule is on.
settings-wallpapers-process-hint = * and ? match
settings-wallpapers-process-action = Choose
settings-wallpapers-process-blank = none yet
settings-wallpapers-resume-hint = After the last rule clears
settings-wallpapers-scope-all = all displays
settings-wallpapers-scope-display = the window's display
settings-wallpapers-rule-fullscreen = fullscreen windows
settings-wallpapers-rule-maximised = maximised windows
settings-wallpapers-rule-focused = focused windows
settings-wallpapers-rule-full-width = full-width columns
settings-wallpapers-plasma-left = is left to Plasma
settings-wallpapers-plasma-static = shows a static image
settings-wallpapers-plasma-follow = follows the last applied wallpaper
settings-wallpapers-plasma-off-row = Unmanaged leaves Plasma alone.
settings-wallpapers-image-hint = Absolute path, or one starting with ~/
settings-wallpapers-image-blank = no image set
settings-wallpapers-dynamic-hint = Poster frame is one silent image
settings-wallpapers-niri-off = Off. The overview keeps niri's own backdrop.
settings-wallpapers-source = Source
settings-wallpapers-look = Look
settings-wallpapers-source-blank = last wallpaper
settings-wallpapers-blur = Blur the backdrop
settings-wallpapers-radii = Blur radius
settings-wallpapers-radii-hint = Images · videos · scenes
settings-wallpapers-dim-hint = 0 none, 100 black
settings-wallpapers-transitions-off = Off. Each wallpaper cuts straight to the next.
settings-wallpapers-transitions-off-foot = Turn transitions on to choose the preview, monitors and sand quality.
settings-wallpapers-awww-none = Images change with { $type }.
settings-wallpapers-awww-no-type = no transition
settings-wallpapers-sound-muted = Wallpaper audio is muted.
settings-wallpapers-displays-say = { $state ->
        [none] No display is locked.
        [one] { $names } is locked.
        [some] { $names } are locked.
       *[detecting] Detecting displays.
    } Displays without their own placement use { $fill }.
settings-wallpapers-play-say = Images use { $backend }, videos use { $val }Vulkan{ $end }, on the { $layer } layer.
settings-wallpapers-transitions-say = { $family } family{ $kind ->
        [random] , a different one every time, over { $duration }.
        [preset] , using { $shader }, in its { $ms } ms timing.
       *[shader] , using { $shader }, over { $duration }.
    } Frame rate { $rate }.
settings-wallpapers-primary-count = { $state ->
        [none] none of the { $all }
        [all] all { $all }
       *[some] { $shown } of { $all }
    }
settings-wallpapers-awww-say = Images play { $type } with the { $filter } filter, over { $duration }.
settings-wallpapers-drawing-say = { $state ->
        [on] Prepared wallpapers draw over { $duration }.
       *[off] Off. Cards set to Draw still draw over { $duration }.
    }
settings-wallpapers-sound-say = Wallpaper audio plays at { $volume }{ $duck ->
        [yes] { " " }and pauses while other apps play sound.
       *[no] .
    }
settings-wallpapers-pause-say = Pauses on { $rules } on { $scope }{ $apps ->
        [yes] , and for listed applications.
       *[no] .
    }
settings-wallpapers-pause-rules = { $count } window rules
settings-wallpapers-plasma-say = { $state ->
        [off] The lock screen { $mode }.
        [static] The lock screen { $mode }: { $image }.
       *[follow] The lock screen { $mode }, scenes as a { $scenes ->
        [live] live render.
       *[poster] poster frame.
    }
    }
settings-wallpapers-niri-say = Shows { $source ->
       *[current] the current wallpaper
        [own] its own source
        [stationary] one stationary wallpaper
    }{ $blur ->
        [yes] , blurred
       *[no] { "" }
    }, dimmed { $dim }{ $auto ->
        [yes] , recoloured
       *[no] { "" }
    }.
settings-wallpapers-device-hint = Automatic tries CUDA, WebGPU, then CPU
settings-wallpapers-memory-hint = Over the limit, Automatic retries on CPU
