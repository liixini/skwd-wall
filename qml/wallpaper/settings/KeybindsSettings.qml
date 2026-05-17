import QtQuick
import "../.."
import "../../components"

Flow {
    id: root
    property var colors

    width: parent ? parent.width : 0
    spacing: 12

    SettingsCard {
        colors: root.colors
        title: "Navigation"
        width: (parent.width - parent.spacing) / 2

        Repeater {
            model: [
                { key: "← / →",      action: "Navigate items" },
                { key: "↑ / ↓",      action: "Navigate rows (hex/grid)" },
                { key: "Enter",      action: "Apply wallpaper" },
                { key: "Escape",     action: "Close panel / overlay" },
                { key: "Right-click", action: "Flip card (details)" },
                { key: "Scroll",     action: "Browse wallpapers" }
            ]
            delegate: SettingsRow {
                colors: root.colors
                title: modelData.key
                description: modelData.action
            }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Filters & tags"
        width: (parent.width - parent.spacing) / 2

        Repeater {
            model: [
                { key: "Shift + ← / →", action: "Cycle colour filters" },
                { key: "Shift + ↑",     action: "Toggle filter bar" },
                { key: "Shift + ↓",     action: "Toggle tag cloud" },
                { key: "Tab",           action: "Auto-complete tag" },
                { key: "Enter",         action: "Add tag (in tag input)" },
                { key: "Escape",        action: "Clear search / close" }
            ]
            delegate: SettingsRow {
                colors: root.colors
                title: modelData.key
                description: modelData.action
            }
        }
    }
}
