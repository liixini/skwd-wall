import QtQuick
import "../.."
import "../../components"
import "../../services"

Flow {
    id: root
    property var colors
    property var saveConfigKey
    property var _recolorThemes: []

    width: parent ? parent.width : 0
    spacing: 12

    Component.onCompleted: {
        DaemonClient.call("effects.list", {}, function(result, err) {
            if (err || !result || !result.effects) return
            for (var i = 0; i < result.effects.length; i++) {
                var eff = result.effects[i]
                if (eff.id !== "theme" || !eff.params) continue
                for (var j = 0; j < eff.params.length; j++) {
                    if (eff.params[j].id === "theme") {
                        root._recolorThemes = eff.params[j].options || []
                        return
                    }
                }
            }
        })
    }

    SettingsCard {
        colors: root.colors
        title: "General"
        width: (parent.width - parent.spacing) / 2

        RowTextInput {
            colors: root.colors
            title: "Monitor"
            description: "Restrict to a specific monitor (e.g. DP-1). Leave empty for all."
            value: Config.mainMonitor
            placeholder: "e.g. DP-1"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("monitor", v) }
        }

        RowDropdown {
            colors: root.colors
            title: "Colour source"
            description: "Whether to use Ollama vision or ImageMagick for palette extraction."
            value: Config.colorSource
            model: [
                { mode: "ollama", label: "Ollama" },
                { mode: "magick", label: "ImageMagick" }
            ]
            onSelect: function(v) { if (root.saveConfigKey) root.saveConfigKey("colorSource", v) }
        }

        RowTextInput {
            colors: root.colors
            title: "Locale"
            description: "City name for the weather filter."
            value: Config.locale
            placeholder: "e.g. London"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("general.locale", v) }
        }

        RowTextInput {
            colors: root.colors
            title: "UI scale"
            description: "Scale the entire selector UI. Range 0.5–2.0 (below 1.0 shrinks it to fit small screens)."
            value: Config.uiScale.toFixed(2)
            placeholder: "1.00"
            onCommit: function(v) {
                var n = parseFloat(v)
                if (isNaN(n)) n = 1.0
                n = Math.max(0.5, Math.min(2.0, n))
                if (root.saveConfigKey) root.saveConfigKey("general.uiScale", n)
            }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Features"
        width: (parent.width - parent.spacing) / 2

        RowToggle {
            colors: root.colors
            title: "Matugen"
            description: "Generate Material 3 colour schemes from the active wallpaper."
            checked: Config.matugenEnabled
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("features.matugen", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Ollama"
            description: "Automated tagging via local LLM."
            checked: Config.ollamaEnabled
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("features.ollama", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Steam Workshop browser"
            description: "Browse and install Wallpaper Engine items from Steam Workshop."
            checked: Config.steamEnabled
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("features.steam", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Wallhaven browser"
            description: "Browse and download wallpapers from wallhaven.cc."
            checked: Config.wallhavenEnabled
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("features.wallhaven", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Mute wallpaper audio"
            description: "Silence video and Wallpaper Engine audio output."
            checked: Config.wallpaperMute
            onToggle: function(v) {
                if (root.saveConfigKey) root.saveConfigKey("wallpaperMute", v)
                DaemonClient.setAudio(v, Config.wallpaperVolume)
            }
        }

        RowInput {
            colors: root.colors
            title: "Wallpaper volume"
            description: "Playback volume for video and Wallpaper Engine audio (0–100%). Has no effect while audio is muted."
            value: Config.wallpaperVolume
            min: 0
            max: 100
            suffix: "%"
            enabled: !Config.wallpaperMute
            onCommit: function(v) {
                if (root.saveConfigKey) root.saveConfigKey("wallpaperVolume", v)
                DaemonClient.setAudio(Config.wallpaperMute, v)
            }
        }

        RowToggle {
            colors: root.colors
            title: "Auto-recolour new wallpapers"
            description: "When a new wallpaper is added, save a gowall theme-recoloured copy alongside it."
            checked: Config.autoRecolorEnabled
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("effects.autoRecolor", v) }
        }

        RowDropdown {
            colors: root.colors
            title: "Recolour theme"
            description: "Palette used when auto-recolouring new wallpapers."
            value: Config.autoRecolorTheme
            model: root._recolorThemes
            enabled: Config.autoRecolorEnabled
            opacity: enabled ? 1.0 : 0.5
            onSelect: function(v) { if (root.saveConfigKey) root.saveConfigKey("effects.autoTheme", v) }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Behaviour"
        width: (parent.width - parent.spacing) / 2

        RowToggle {
            colors: root.colors
            title: "Apply per monitor"
            description: "Allow picking a different wallpaper for each monitor. Video and Wallpaper Engine support is in progress."
            checked: Config.wallpaperPerMonitor
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("general.wallpaperPerMonitor", v) }
        }

        RowInput {
            colors: root.colors
            title: "Selector backdrop dim (%)"
            description: "How dark to make the area behind the wallpaper selector card. 0 disables the dim entirely."
            value: Config.selectorBackdropOpacity
            min: 0; max: 100
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("general.selectorBackdropOpacity", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Close on selection"
            description: "Hide the selector immediately after applying a wallpaper."
            checked: Config.closeOnSelection
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("general.closeOnSelection", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Always show filter bar"
            description: "Keep the filter bar pinned visible instead of auto-hiding."
            checked: Config.filterBarAlwaysVisible
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("general.filterBarAlwaysVisible", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Always show search bar"
            description: "Keep the search bar pinned visible."
            checked: Config.searchBarAlwaysVisible
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("general.searchBarAlwaysVisible", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Video auto-scale"
            description: "Auto-scale videos to fit the wallpaper resolution."
            checked: Config.videoAutoScale
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("features.videoAutoScale", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Notify on wallpaper change"
            description: "Send a system notification each time the wallpaper changes."
            checked: Config.notifyOnWallpaperChange
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("general.notifyOnWallpaperChange", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Restore wallpaper on startup"
            description: "Re-apply the last wallpaper when the daemon starts."
            checked: Config.restoreOnStartup
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("restoreOnStartup", v) }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Random rotation"
        width: (parent.width - parent.spacing) / 2

        RowInput {
            colors: root.colors
            title: "Interval"
            description: "Seconds between random rotations."
            value: Config.randomInterval
            min: 1; max: 86400
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("general.randomInterval", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Include images"
            description: "Allow static wallpapers in the random pool."
            checked: Config.randomIncludeStatic
            onToggle: function(v) {
                if (!v && !Config.randomIncludeVideo && !Config.randomIncludeWE) return
                if (root.saveConfigKey) root.saveConfigKey("general.randomIncludeStatic", v)
            }
        }

        RowToggle {
            colors: root.colors
            title: "Include video"
            description: "Allow video wallpapers in the random pool."
            checked: Config.randomIncludeVideo
            onToggle: function(v) {
                if (!v && !Config.randomIncludeStatic && !Config.randomIncludeWE) return
                if (root.saveConfigKey) root.saveConfigKey("general.randomIncludeVideo", v)
            }
        }

        RowToggle {
            colors: root.colors
            title: "Include Wallpaper Engine"
            description: "Allow Wallpaper Engine items in the random pool."
            checked: Config.randomIncludeWE
            onToggle: function(v) {
                if (!v && !Config.randomIncludeStatic && !Config.randomIncludeVideo) return
                if (root.saveConfigKey) root.saveConfigKey("general.randomIncludeWE", v)
            }
        }

        RowToggle {
            colors: root.colors
            title: "Favourites only"
            description: "Restrict the random pool to favourited wallpapers."
            checked: Config.randomIncludeFavourites
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("general.randomIncludeFavourites", v) }
        }
    }
}
