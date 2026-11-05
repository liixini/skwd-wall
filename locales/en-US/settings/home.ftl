settings-home-search = Search every setting: frame rate, sound, lock screen, blur
settings-home-displays = { $count ->
    [one] { $count } display
   *[other] { $count } displays
 }
settings-home-displays-say = { $link } { $count ->
    [one] is
   *[other] are
 } showing wallpapers.
settings-home-watching = { " " }Library watching: { $val }{ $state }{ $end }.
settings-home-model-ready = ready
settings-home-model-missing = no model
settings-home-model-off = off
settings-home-colours-from = { " " }{ $link } come from { $val }{ $engine }{ $end }.
settings-home-sound-desc = Audio from video and Wallpaper Engine wallpapers.
settings-home-sound-say = Wallpapers play at { $volume }{ $duck ->
    [yes] { " " }and pause while other apps play sound.
   *[no] , even while other apps play sound.
 }
settings-home-colours-desc = How wallpaper colours reach the rest of the desktop.
settings-home-colours-say = Palette behaviour is { $policy }.{ " " }More in { $place }.
settings-home-rotation-desc = Apply a random wallpaper whenever the interval expires.
settings-home-rotation-say = A random wallpaper every { $interval }.
settings-home-rotation-off = Rotation is off. The wallpaper only changes when you pick one.
settings-home-transitions-desc = How one wallpaper changes into the next.
settings-home-transitions-say = One wallpaper changes into the next over { $duration }.
settings-home-transitions-off = Transitions are off. Wallpapers cut straight to the next one.
settings-home-changed = Changed from default
settings-home-changed-desc = Settings you changed from their defaults, each with Reset.
settings-home-changed-none = Nothing differs from its default among the settings compared here.
settings-home-changed-say = { $count ->
    [one] { $count } setting differs from its default.
   *[other] { $count } settings differ from their defaults.
 }
settings-home-review = Review
settings-home-reset = Reset
settings-home-reset-all = Reset all
settings-home-badge = { $count } changed
settings-home-automatic = Automatic
settings-home-displays-none = No displays
settings-home-displays-none-say = { $link } are showing wallpapers.
settings-home-reset-scope = Resets every setting above, keybindings included. API keys, usernames, locations and folders stay.
settings-home-items = { $count ->
    [one] { $count } item
   *[other] { $count } items
 }
