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
            value: Config.wallhavenColumns
            min: 2; max: 12
            onCommit: function(v) { if (root.saveField) root.saveField("wallhavenColumns", v) }
        }

        RowInput {
            colors: root.colors
            title: "Rows"
            description: "Number of rows visible at once."
            value: Config.wallhavenRows
            min: 1; max: 10
            onCommit: function(v) { if (root.saveField) root.saveField("wallhavenRows", v) }
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
            value: Config.wallhavenThumbWidth
            min: 100; max: 600
            onCommit: function(v) { if (root.saveField) root.saveField("wallhavenThumbWidth", v) }
        }

        RowInput {
            colors: root.colors
            title: "Height"
            description: "Thumbnail height in pixels."
            value: Config.wallhavenThumbHeight
            min: 60; max: 600
            onCommit: function(v) { if (root.saveField) root.saveField("wallhavenThumbHeight", v) }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "API"
        width: parent.width

        RowTextInput {
            colors: root.colors
            title: "API key"
            description: "Wallhaven API key (required for NSFW content)."
            value: Config.wallhavenApiKey
            placeholder: "Wallhaven API key (for NSFW)"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("wallhaven.apiKey", v) }
        }
    }
}
