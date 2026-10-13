playlists-unnamed = (unnamed)
playlists-generated-name = Playlist { $number }
playlists-new-name = New playlist name
playlists-name = Playlist name
playlists-play-now = Play now
playlists-delete = Delete
playlists-type-curated = Manual
playlists-type-smart = Filtered
playlists-order-shuffle = Shuffle
playlists-order-sequential = Sequential
playlists-every = Every
playlists-seconds-minimum = seconds (min 5)
playlists-all = All
playlists-favourites = Favourites
playlists-filter-example = e.g. type:video color:blue tag:cat,-anime
playlists-empty-smart = No wallpapers match this filter yet.
playlists-empty-curated = This playlist is empty. Open a wallpaper card and choose Playlist to add it.
playlists-stop-all = ■ Stop all
playlists-card-add-title = Add to playlist
playlists-card-none = No playlists exist yet. Name one to create it with this wallpaper already added:
playlists-card-new = Or create another playlist with this wallpaper:
playlists-masthead = Playlists  /  { $selected }
playlists-masthead-fallback = Library
playlists-index-title = Playlists
playlists-index-subtitle = A manual playlist contains wallpapers you choose. A filtered playlist stays in sync with its rules.
playlists-index-live-detail = { $kind } · active on { $outputs }
playlists-index-count-detail =
    { $kind }  ·  { $count } { $count ->
        [one] wallpaper
       *[other] wallpapers
    }
playlists-index-empty-title = No playlists yet
playlists-index-empty-hint = Create one below to begin.
playlists-new-playlist = New playlist
playlists-live-routing = Active displays
playlists-active-assignments =
    { $count } active { $count ->
        [one] assignment
       *[other] assignments
    }
playlists-library = Saved playlists
playlists-library-stats = { $count } saved · none active
playlists-live-results = Matching wallpapers
playlists-the-sequence = Playback order
playlists-sequence-smart-hint = Wallpapers matching the current filter, in the order shown.
playlists-sequence-curated-hint = Wallpapers play from top to bottom. Drag them to change the order.
playlists-empty-smart-hint = No wallpapers match these rules. Change the filter to include more.
playlists-empty-curated-hint = Open a wallpaper card and choose Playlist to add the first wallpaper.
playlists-live-badge = Active
playlists-empty-eyebrow = Playlists / library
playlists-empty-title = Build a rotation
playlists-empty-choose = Choose a playlist from the index
playlists-empty-desc = Or create a new manual playlist and start adding wallpapers.
playlists-kind-smart = Filtered playlist
playlists-kind-curated = Manual playlist
playlists-state-live = Active / { $outputs }
playlists-state-query-ready = Filter ready
playlists-state-ready =
    { $padded } { $count ->
        [one] wallpaper
       *[other] wallpapers
    } ready
playlists-edition-eyebrow = Playlists / edit
playlists-kind-tagline = { $kind } · ready for display assignment
playlists-stop-edition = Stop playlist
playlists-colour-notes = Colours
playlists-folder-notes = Folders
playlists-live-collection-title = Filtered playlist
playlists-live-collection-desc = This list updates itself from the selected source, tags, resolution, and colour filters.
playlists-definition-title = Playlist details
playlists-definition-desc = Set the name and choose whether wallpapers are added manually or selected by a filter.
playlists-playback-title = Playback
playlists-playback-desc = Choose the order and how long each wallpaper stays on screen.
playlists-routing-title = Apply to displays
playlists-routing-desc = Choose where this playlist runs. Selecting every display creates one shared assignment.
playlists-routing-all = Every connected display
playlists-routing-live = Active assignment
playlists-routing-none = No active assignment
playlists-demo-name = The baddest cut

playlists-id = ID { $id }
playlists-picker-entry = { $name } · ID { $id }
playlists-choose = Choose a playlist
playlists-browse-all = All wallpapers
playlists-edit = Edit playlists
playlists-picker-search = Search by name or ID
playlists-picker-no-match = No playlists match this search.
playlists-picker-hint = Choose which wallpapers to browse. Playback stays as it is.
playlists-filter-help = Filter reference
playlists-help-combine = Separate filters with spaces. Every filter must match, except folder rules: an item in any listed folder counts. Quote paths that contain spaces.
playlists-help-source = all / favourites
    Start with all wallpapers or only favourites.
playlists-help-folder = folder:nature/forest
    Match this folder and its subfolders. Several folder rules match any of them; quote paths that contain spaces.
playlists-help-type = type:image / type:video / type:we
    Images, videos, or Wallpaper Engine wallpapers. Also accepts static, picture, pic, img; vid, mp4; scene, wallpaperengine, wallpaper-engine.
playlists-help-tag = tag:cat,night / tag:cat|dog,-anime
    Commas require every tag; | accepts either tag; - excludes a tag. Include at least one positive tag. Tags ignore letter case.
playlists-help-colour = color:blue,cyan / colour:gray
    Commas accept any listed colour. Colours: red, orange, yellow, lime, green, spring, cyan, azure, blue, violet, magenta, pink, gray. Numeric buckets 0-11 and 99 also work.
    Aliases: chartreuse; mint, emerald; teal, aqua; sky; purple, indigo; fuchsia; rose; grey, mono, monochrome, grayscale, greyscale, black, white.
playlists-help-ratio = ratio:portrait / ratio:landscape / ratio:square
    Tall, wide, or nearly square (within 0.05 of 1:1). Use decimal ratios for comparisons, such as ratio:>1.7.
playlists-help-width = width:>=1920
    Compare the wallpaper width in pixels.
playlists-help-height = height:>=1080
    Compare the wallpaper height in pixels.
playlists-help-resolution = res:>=1920x1080
    Both dimensions must satisfy the comparison.
playlists-help-comparison = Comparisons: >=, <=, >, <, =
    With no operator, numbers mean at least. Numeric = allows a difference below 0.5, including ratios. Use lowercase filter keywords.
playlists-help-example = Example: favourites type:video ratio:landscape color:blue
    Blue, wide videos from your favourites.
