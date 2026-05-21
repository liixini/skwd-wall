import QtQuick
import "../.."
import "../../components"

Column {
    id: root
    property var colors
    property var saveConfigKey
    property var notifyThemeChanged

    width: parent ? parent.width : 0
    spacing: 8

    SettingsCard {
        colors: root.colors
        title: "Theme"
        width: parent.width

        RowDropdown {
            colors: root.colors
            title: "Scheme type"
            description: "Material 3 colour-generation algorithm."
            value: Config.matugenScheme.replace("scheme-", "")
            model: [
                { mode: "content",     label: "Content" },
                { mode: "expressive",  label: "Expressive" },
                { mode: "fidelity",    label: "Fidelity" },
                { mode: "fruit-salad", label: "Fruit salad" },
                { mode: "monochrome",  label: "Monochrome" },
                { mode: "neutral",     label: "Neutral" },
                { mode: "rainbow",     label: "Rainbow" },
                { mode: "tonal-spot",  label: "Tonal spot" },
                { mode: "vibrant",     label: "Vibrant" }
            ]
            onSelect: function(v) {
                var full = "scheme-" + v
                if (root.saveConfigKey) root.saveConfigKey("matugen.schemeType", full)
                if (root.notifyThemeChanged) root.notifyThemeChanged(full, Config.matugenMode, Config.matugenColorIndex)
            }
        }

        RowDropdown {
            colors: root.colors
            title: "Mode"
            description: "Dark or light theme."
            value: Config.matugenMode
            model: [
                { mode: "dark",  label: "Dark" },
                { mode: "light", label: "Light" }
            ]
            onSelect: function(v) {
                if (root.saveConfigKey) root.saveConfigKey("matugen.mode", v)
                if (root.notifyThemeChanged) root.notifyThemeChanged(Config.matugenScheme, v, Config.matugenColorIndex)
            }
        }

        RowDropdown {
            colors: root.colors
            title: "Source colour index"
            description: "Which palette slot to use as the seed colour."
            value: String(Config.matugenColorIndex)
            model: [
                { mode: "0", label: "0 (Primary)" },
                { mode: "1", label: "1" },
                { mode: "2", label: "2" },
                { mode: "3", label: "3" }
            ]
            onSelect: function(v) {
                var idx = parseInt(v, 10) | 0
                if (root.saveConfigKey) root.saveConfigKey("matugen.colorIndex", idx)
                if (root.notifyThemeChanged) root.notifyThemeChanged(Config.matugenScheme, Config.matugenMode, idx)
            }
        }
    }
}
