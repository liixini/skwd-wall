import QtQuick
import "../.."
import "../../components"

Flow {
    id: root
    property var colors
    property var saveConfigKey

    width: parent ? parent.width : 0
    spacing: 12

    SettingsCard {
        colors: root.colors
        title: "Directories"
        width: parent.width

        RowTextInput {
            colors: root.colors
            title: "Wallpaper directory"
            description: "Folder Skwd scans for image and video wallpapers. Restart required after change."
            value: Config.wallpaperDir
            placeholder: "~/Pictures/Wallpapers"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("paths.wallpaper", v) }
        }

        RowTextInput {
            colors: root.colors
            title: "Video directory"
            description: "Separate folder for video wallpapers. Defaults to the wallpaper directory."
            value: Config.videoDir
            placeholder: "(same as wallpaper directory)"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("paths.videoWallpaper", v) }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Steam"
        width: parent.width

        RowTextInput {
            colors: root.colors
            title: "Workshop directory"
            description: "Where Steam stores Workshop content."
            value: Config.weDir
            placeholder: "Steam Workshop content path"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("paths.steamWorkshop", v) }
        }

        RowTextInput {
            colors: root.colors
            title: "WE assets directory"
            description: "Wallpaper Engine assets path."
            value: Config.weAssetsDir
            placeholder: "Wallpaper Engine assets path"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("paths.steamWeAssets", v) }
        }

        RowTextInput {
            colors: root.colors
            title: "Steam directory"
            description: "Steam install root."
            value: Config.steamDir
            placeholder: "Steam install path"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("paths.steam", v) }
        }
    }
}
