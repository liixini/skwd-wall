settings-library-folders-desc = Scan these folders and their subfolders for images and videos.
settings-library-folders-say = Scanning { $val }{ $folders }{ $end }.{ $state ->
        [polling] { " " }Polling covers what native watching misses, every { $every }.{ $unread }
        [native] { " " }Native file watching is active; polling is idle.
        [down] { " " }Native watching failed and polling is off.
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }One folder cannot be read.
       *[other] { " " }{ $count } folders cannot be read.
    }
settings-library-watching = Watching
settings-library-poll-hint = For network or FUSE folders
settings-library-interval-hint = Restart skwd-walld after changing it
settings-library-excluded-desc = Skip these folders and their subfolders. Existing metadata stays saved.
settings-library-excluded-say = Never scanning { $val }{ $folders }{ $end }.{ $outside }
settings-library-outside = { $count ->
        [one] { " " }One is outside every wallpaper folder and skips nothing.
       *[other] { " " }{ $count } are outside every wallpaper folder and skip nothing.
    }
settings-library-excluded-hint = Excluded folders are never scanned, so no profile can show them. To keep a folder but hide it in public, use { $link }.
settings-library-profiles-desc = Decide which folders the picker, the folder filter, random wallpapers and schedules use.
settings-library-profiles-say = Active profile { $state ->
        [none] { $menu }, which shows every folder.
       *[some] { $menu }: { $summary }.
    } Ctrl+H or the bar's hidden toggle shows hidden wallpapers, including ones hidden from their card, until the next launch.
settings-library-active = Active profile
settings-library-terminal = Switch from a terminal
settings-library-tree-note = Hide stays out of random until Ctrl+H. Exclude stays out even with Ctrl+H. A folder sets its subfolders.
settings-library-show-tip = In the picker and random
settings-library-hide-tip = Until Ctrl+H, never random
settings-library-exclude-tip = Left out of this profile, Ctrl+H skips it
settings-library-from = From { $link }
settings-library-excluded-inside = { $count } excluded inside
settings-library-both-inside = { $hidden } hidden, { $excluded } excluded inside
settings-library-storage = Storage and trash
settings-library-storage-desc = Smaller WebP copies, recovery storage and recoloured copies.
settings-library-storage-say = { $optimise ->
        [on] New images become WebP at { $quality } quality, capped at { $size }.
       *[off] New images stay as they are. Optimise all uses { $quality } quality, capped at { $size }.
    } { $trash ->
        [on] Trashed originals stay for { $days }, then are deleted.
       *[off] Trashed originals stay until you delete them.
    } { $recolour ->
        [on] New wallpapers also get a recoloured copy.
       *[off] New wallpapers are not recoloured.
    }
settings-library-retention-hint = Days to keep trashed images
settings-library-optimise-hint = Run image optimisation across the library
settings-library-recolour = Recolour
settings-library-thumbnails = Thumbnails and repair
settings-library-thumbnails-desc = Rebuild previews and colour data when something looks wrong.
settings-library-thumbnails-say = Up to { $jobs } thumbnails are made at once during rebuilds.
settings-library-rebuild = Rebuild
settings-library-start-over = Start over
settings-library-recompute-hint = From existing thumbnails
settings-library-clear-hint = Thumbnails regenerate on the next scan
