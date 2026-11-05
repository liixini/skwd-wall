settings-library-watch-section-desc = Detect files added outside skwd-wall and keep the library current.
settings-library-watch-fallback-label = Polling fallback
settings-library-watch-fallback-desc = Check only library folders that native file watching cannot watch. Enable this for network or FUSE mounts that miss changes, then restart skwd-walld.
settings-library-watch-interval-label = Polling interval
settings-library-watch-interval-desc = Wait this many seconds between bounded checks. Lower values find changes sooner but read the filesystem more often. Restart skwd-walld after changing it.
settings-library-watch-unknown-label = Watcher status unavailable
settings-library-watch-unknown-desc = This daemon does not report library-watch status. Update or restart skwd-walld.
settings-library-watch-poll-failed-label = Polling cannot read a library folder
settings-library-watch-poll-failed-desc = Check that every configured library folder is mounted and readable. Polling will retry in { $interval } seconds.
settings-library-watch-polling-label = Polling fallback active
settings-library-watch-polling-desc = Native watching failed for { $count ->
    [one] one library folder
   *[other] { $count } library folders
    }. Up to { $budget } entries are checked every { $interval } seconds. Last successful convergence: { $convergence }.
settings-library-watch-recovering-label = Native watching recovered
settings-library-watch-recovering-desc = The native watcher is active again. A full hand-off scan is still running before the library is declared current.
settings-library-watch-unavailable-label = Library watching unavailable
settings-library-watch-unavailable-desc = Native file watching failed and polling fallback is off. Enable Polling fallback, then restart skwd-walld.
settings-library-watch-recovered-label = Native watching restored
settings-library-watch-recovered-desc = The native watcher and its hand-off scan are current. Last successful convergence: { $convergence }.
settings-library-watch-native-label = Native file watching
settings-library-watch-native-desc = Filesystem events are active for every library folder. Polling is idle.
settings-library-watch-convergence-never = Not completed yet
settings-library-watch-convergence-seconds = { $value ->
    [one] 1 second ago
   *[other] { $value } seconds ago
    }
settings-library-watch-convergence-minutes = { $value ->
    [one] 1 minute ago
   *[other] { $value } minutes ago
    }
settings-library-watch-convergence-hours = { $value ->
    [one] 1 hour ago
   *[other] { $value } hours ago
    }
settings-profiles-card = Profiles
settings-profiles-card-desc = Decide which folders the picker, the folder filter, random wallpapers and schedules use. Hidden folders come back with the hidden toggle in the bar or its keybind, and every launch starts with them hidden again.
settings-profiles-everything = Everything
settings-profiles-name-label = Name
settings-profiles-name-placeholder = Profile name
settings-profiles-state-show = Show
settings-profiles-state-hide = Hide
settings-profiles-state-exclude = Exclude
settings-profiles-workshop = Wallpaper Engine
settings-profiles-workshop-desc = Workshop items have no folders, so this sets all of them. Hide a single wallpaper from the back of its card.
settings-profiles-summary = { $hidden } hidden, { $excluded } excluded
settings-profiles-summary-none = Shows every folder
settings-profiles-delete-action = Delete profile
settings-profiles-add-label = New profile
settings-profiles-new-name = New profile
settings-profiles-never-scanned = Never scanned
settings-profiles-mixed = Mixed inside
settings-profiles-hidden-inside = { $count ->
    [one] { $count } hidden inside
   *[other] { $count } hidden inside
    }
settings-profiles-copy-action = Copy
settings-profiles-helm-desc = Makes this the active profile, so rotation and schedules follow.
settings-profiles-flag-desc = Opens one picker in this profile and leaves the active one alone.
settings-profiles-make-active = Make active
settings-profiles-active-tag = Active
settings-profiles-builtin-tag = Built in
settings-profiles-filter-placeholder = Filter folders
settings-profiles-no-match = No folder matches.
settings-profiles-hidden-note-label = Hidden wallpapers
settings-profiles-everything-desc = Everything shows every scanned folder and every Workshop item. Make a new profile to hide or exclude some of them.
settings-profiles-delete-hint = Click again to delete. Wallpapers and folders stay where they are.
