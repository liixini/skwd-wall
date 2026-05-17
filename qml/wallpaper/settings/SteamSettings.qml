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
        title: "API"
        width: parent.width

        RowTextInput {
            colors: root.colors
            title: "API key"
            description: "Steam Web API key."
            value: Config.steamApiKey
            placeholder: "Steam API key"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("steam.apiKey", v) }
        }

        RowTextInput {
            colors: root.colors
            title: "Username"
            description: "Used by steamcmd for Workshop downloads."
            value: Config.steamUsername
            placeholder: "Steam username (for steamcmd)"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("steam.username", v) }
        }
    }
}
