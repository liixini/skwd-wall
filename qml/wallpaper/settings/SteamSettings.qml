import QtQuick
import "../.."
import "../../components"

Flow {
    id: root
    property var colors
    property var saveField
    property var saveConfigKey

    width: parent ? parent.width : 0
    spacing: 12

    SettingsCard {
        colors: root.colors
        title: "Grid"
        width: (parent.width - parent.spacing) / 2

        RowInput {
            colors: root.colors
            title: "Columns"
            description: "Number of thumbnails per row."
            value: Config.steamColumns
            min: 2; max: 12
            onCommit: function(v) { if (root.saveField) root.saveField("steamColumns", v) }
        }

        RowInput {
            colors: root.colors
            title: "Rows"
            description: "Number of rows visible at once."
            value: Config.steamRows
            min: 1; max: 10
            onCommit: function(v) { if (root.saveField) root.saveField("steamRows", v) }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Thumbnail"
        width: (parent.width - parent.spacing) / 2

        RowInput {
            colors: root.colors
            title: "Width"
            description: "Thumbnail width in pixels."
            value: Config.steamThumbWidth
            min: 100; max: 600
            onCommit: function(v) { if (root.saveField) root.saveField("steamThumbWidth", v) }
        }

        RowInput {
            colors: root.colors
            title: "Height"
            description: "Thumbnail height in pixels."
            value: Config.steamThumbHeight
            min: 60; max: 600
            onCommit: function(v) { if (root.saveField) root.saveField("steamThumbHeight", v) }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Download backend"
        width: parent.width

        RowDropdown {
            colors: root.colors
            title: "Backend"
            description: "Two ways to set up Steam Workshop:\n- API key + steamcmd (recommended): add a Steam Web API key and set the backend to steamcmd. Browsing and downloads both work, no running Steam needed.\n- Steam Client: leave defaults and keep Steam running. Native client support on NixOS and Flatpak Steam is WIP - use the API key + steamcmd setup there.\n\nSwitching backend takes effect after the daemon restarts."
            value: Config.steamBackend
            model: [
                { mode: "steamcmd", label: "steamcmd + API key (recommended)" },
                { mode: "steam",    label: "Steam Client" }
            ]
            onSelect: function(v) { if (root.saveConfigKey) root.saveConfigKey("steam.backend", v) }
        }

        RowTextInput {
            colors: root.colors
            title: "Username"
            description: "Only used by the steamcmd backend."
            value: Config.steamUsername
            placeholder: "Steam username (for steamcmd)"
            enabled: Config.steamBackend === "steamcmd"
            opacity: enabled ? 1.0 : 0.5
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("steam.username", v) }
        }

        RowTextInput {
            colors: root.colors
            title: "API key"
            description: "Only used by the steamcmd backend, for browsing the Workshop catalogue inside skwd-wall. The Steam Client backend gets browse access for free from the running Steam session."
            value: Config.steamApiKey
            placeholder: "Steam API key"
            enabled: Config.steamBackend === "steamcmd"
            opacity: enabled ? 1.0 : 0.5
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("steam.apiKey", v) }
        }
    }
}
