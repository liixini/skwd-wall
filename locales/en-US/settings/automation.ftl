settings-automation-rotation-desc = Apply a random wallpaper from the pool whenever the interval expires.
settings-automation-rotation-say = Every { $interval }, a random pick from { $val }{ $pool }{ $end }.
settings-automation-rotation-off = Off. The wallpaper only changes when you pick one.
settings-automation-pool = { $count ->
        [none] an empty pool
        [one] { $first }
        [two] { $first } and { $second }
       *[three] { $first }, { $second } and { $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] favourite { $pool }
       *[all] { $pool }
    }
settings-automation-pool-images = images
settings-automation-pool-videos = videos
settings-automation-pool-engine = Wallpaper Engine items
settings-automation-pool-heading = The pool holds
settings-automation-timing = Timing
settings-automation-interval-hint = minimum 10 s
settings-automation-schedule-desc = Rules pick the wallpaper and theme from time, weekday, weather, power and displays. The first matching rule wins.
settings-automation-schedule-say = { $start ->
        [applies] Rules are on; the first matching rule { $val }applies on startup{ $end }. A wallpaper you pick yourself still wins.
       *[waits] Rules are on; the first matching rule { $val }waits for the next change{ $end }. A wallpaper you pick yourself still wins.
    }
settings-automation-schedule-off = Off. Your rules are kept until you turn this on.
settings-automation-start-hint = first matching rule when the service starts
settings-automation-editor-hint = build rules from condition blocks
settings-automation-location = Location and weather
settings-automation-location-desc = Used for sunrise and sunset times and for weather conditions.
settings-automation-location-say = Weather for { $city }, sun times at { $lat }, { $lon }. Weather changes: { $mode }.
settings-automation-no-city = no city
settings-automation-no-latitude = no latitude
settings-automation-no-longitude = no longitude
settings-automation-location-hint = Leave the city empty to use the coordinates instead. Latitude is north positive, longitude east positive.
settings-automation-startup = Startup and notices
settings-automation-startup-desc = What happens when the service starts and when a wallpaper changes.
settings-automation-startup-say = { $restore ->
        [restores] On startup Skwd { $val }re-applies the last wallpaper{ $end }.
       *[leaves] On startup Skwd { $val }leaves the desktop as it is{ $end }.
    } { $notify ->
        [notifies] Wallpaper changes { $val }send a notification{ $end }.
       *[silent] Wallpaper changes { $val }stay silent{ $end }.
    }
settings-automation-commands = After a wallpaper applies
settings-automation-commands-desc = Run your own commands after every change, for example to drive another setter.
settings-automation-command-count = { $count ->
        [one] { $val }{ $count } command{ $end } runs after each change.
       *[other] { $val }{ $count } commands{ $end } run after each change.
    }
settings-automation-commands-say = { $listed ->
        [none] { $val }No commands{ $end } run after each change.
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] Skwd { $val }leaves applying to your commands{ $end }, and the commands { $val }also run on the startup restore{ $end }.
           *[skip] Skwd { $val }leaves applying to your commands{ $end }, and the commands { $val }skip the startup restore{ $end }.
        }
       *[itself] { $rerun ->
            [also] Skwd { $val }applies the wallpaper itself{ $end }, and the commands { $val }also run on the startup restore{ $end }.
           *[skip] Skwd { $val }applies the wallpaper itself{ $end }, and the commands { $val }skip the startup restore{ $end }.
        }
    }
settings-automation-pick-only-hint = your commands apply it
settings-automation-commands-heading = Commands
settings-automation-placeholders = Use %path%, %thumb%, %type% and %name% in a command.
settings-automation-minutes = { $count } min
settings-automation-mode-off = Off
settings-automation-mode-outdoors = Follow the weather outdoors
settings-automation-mode-playlists = Pick from your playlists
settings-automation-mode-semantic = Search your library
settings-automation-condition-clear = Clear
settings-automation-condition-cloudy = Cloudy
settings-automation-condition-rainy = Rainy
settings-automation-condition-snowy = Snowy
settings-automation-condition-stormy = Stormy
settings-automation-condition-foggy = Foggy
settings-automation-condition-windy = Windy
settings-automation-display-another = Another
settings-automation-display-exclude = Exclude
settings-automation-display-keep = Keep
settings-automation-playlists-heading = Playlist per weather
settings-automation-hours = { $count } h
