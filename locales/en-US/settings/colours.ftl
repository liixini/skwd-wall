settings-colours-source = Where colour comes from
settings-colours-source-desc = Follow each wallpaper, keep one palette, or leave colours alone.
settings-colours-source-follow = follow each wallpaper
settings-colours-source-fixed = come from one fixed palette
settings-colours-source-off = are left as they are
settings-colours-source-say = Colours { $policy }.
settings-colours-source-made = Colours { $policy }, made by { $authority }.
settings-colours-end4 = end-4
settings-colours-control-menu = Colour control · detected shells
settings-colours-generator-menu = Generator · installed only
settings-colours-theme-menu = Theme · presets, yours, custom seeds
settings-colours-all-presets = All { $count } presets…
settings-colours-generator = Generator
settings-colours-generator-desc = The program that turns the wallpaper into a palette.
settings-colours-generator-say = { $engine } makes the palette.
settings-colours-generator-seeded = { $engine } makes the palette, seeded from slot { $index } at contrast { $contrast }.
settings-colours-generator-shell = { $shell } makes the palette from the wallpaper.
settings-colours-palette = Palette
settings-colours-palette-desc = One palette for every wallpaper.
settings-colours-palette-say = { $theme } for every wallpaper.
settings-colours-idle = Not used while colours are left as they are.
settings-colours-index-hint = Which palette slot to use as the seed colour
settings-colours-contrast-hint = -1 to 1
settings-colours-wallust-hint = Force a wallust palette, e.g. dark16. Empty is auto by mode
settings-colours-seeds-hint = Used by Custom seeds
settings-colours-config-hint = External Matugen
settings-colours-command-hint = %config% %path% %scheme% %mode% %index%
settings-colours-auto = Auto
settings-colours-look = Look
settings-colours-look-desc = How strongly colour spreads, its finish, and light or dark.
settings-colours-look-say = { $scheme } character with a { $finish } finish, variant { $variant }.
settings-colours-look-imported = { $shell }'s palette, imported as the { $variant } variant.
settings-colours-look-shell = { $shell } sets its own look.
settings-colours-look-fixed = The fixed palette sets its own look.
settings-colours-no-designer = The theme designer is available with Follow wallpaper or Fixed palette.
settings-colours-designer-hint = Tune or build a palette by hand
settings-colours-go = Where colours go
settings-colours-go-desc = App themes that follow the palette when it changes.
settings-colours-find-hint = Check again after installing an app
settings-colours-matugen-hint = Feature switch for Matugen outputs
settings-colours-reload-hint = Run after the file is written
settings-colours-output-heading = Output { $count }
settings-colours-shells = Shells
settings-colours-shells-desc = Noctalia and Dank Material Shell can take Skwd's colours.
settings-colours-dms-hint = DMS must use Auto (wallpaper)
settings-colours-mode-hint = Applied with each colour scheme
settings-colours-bin-hint = If it is not on PATH
settings-colours-lock = Lock screens
settings-colours-lock-desc = Lock screens that show your wallpaper.
settings-colours-lock-follow = the last applied wallpaper
settings-colours-lock-still = a static image
settings-colours-sddm-hint = Needs administrator approval
settings-colours-go-say = { $val }{ $on } of { $all } apps{ $end } { $followers ->
        [one] follows
       *[some] follow
    } your colours{ $changed ->
        [one] ; { $val }{ $name }'s config changed outside Skwd{ $end }.
        [some] ; { $val }{ $count } apps changed outside Skwd{ $end }.
       *[none] .
    }
settings-colours-outputs-say = { $state ->
        [none] No integration renders a template yet.
        [single] { $val }1 integration{ $end } renders a template to a file after each colour change.
       *[many] { $val }{ $count } integrations{ $end } render templates to files after each colour change.
    }
settings-colours-shells-say = Noctalia { $val }{ $noctalia ->
        [follows] follows Skwd colours
       *[keeps] keeps its own colours
    }{ $end }; DMS { $val }{ $dms ->
        [follows] follows Skwd colours
       *[keeps] keeps its own colours
    }{ $end }{ $caelestia ->
        [yes] ; Caelestia wallpapers sync.
       *[no] .
    }
settings-colours-lock-say = { $state ->
        [none] No lock screen is set up. New ones use { $mode }.
        [one] { $names } uses { $mode }.
       *[some] { $names } use { $mode }.
    }
settings-colours-lock-say-static = { $state ->
        [none] No lock screen is set up. New ones use { $mode }: { $image }.
        [one] { $names } uses { $mode }: { $image }.
       *[some] { $names } use { $mode }: { $image }.
    }
settings-menu-more = More
