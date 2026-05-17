import QtQuick
import "../.."
import "../../components"

Column {
    id: root
    property var colors
    property var service

    width: parent ? parent.width : 0
    spacing: 8

    SettingsCard {
        colors: root.colors
        title: "Sort mode"
        subtitle: "How wallpapers are ordered in the selector. The same option is also available as a quick toggle in the filter bar."

        Repeater {
            model: [
                { mode: "color",       label: "Default",      description: "Group by dominant color (red - orange - yellow - … - pink - neutral). Within each color, the most-saturated wallpapers come first." },
                { mode: "pop",         label: "Color pop",    description: "Sort by how much vivid color the wallpaper actually contains, ignoring which color it is. Saturated, eye-catching wallpapers lead; muted and mostly-neutral ones drop to the back." },
                { mode: "richness",    label: "Colourful",    description: "Sort by palette diversity - wallpapers using many distinct colors lead, monochrome scenes fall to the back. Good for finding maximalist artwork and detailed illustrations." },
                { mode: "minimalist",  label: "Minimalist",   description: "Inverse of richness - fewest colors first. Solid backgrounds, clean gradients, and single-subject compositions surface; busy multi-color images drop to the back." },
                { mode: "applied",     label: "Most applied", description: "Sorts by how often you've actually set this wallpaper. Your greatest hits lead; never-applied wallpapers fall back to newest-first within them." },
                { mode: "date",        label: "Newest",       description: "Most recently added wallpapers first. Useful right after dropping new files into your wallpaper directory." }
            ]
            delegate: SettingsRow {
                colors: root.colors
                title: modelData.label
                description: modelData.description
                onClicked: {
                    if (!root.service) return
                    root.service.sortMode = modelData.mode
                    root.service.updateFilteredModel()
                }

                Rectangle {
                    width: 16; height: 16; radius: 8
                    anchors.verticalCenter: parent.verticalCenter
                    property bool active: root.service && root.service.sortMode === modelData.mode
                    color: active
                        ? (root.colors ? root.colors.primary : "#7986cb")
                        : "transparent"
                    border.width: 2
                    border.color: root.colors ? root.colors.primary : "#7986cb"
                    Behavior on color { ColorAnimation { duration: 120 } }
                    Rectangle {
                        anchors.centerIn: parent
                        width: 6; height: 6; radius: 3
                        color: root.colors ? root.colors.primaryText : "#000"
                        visible: parent.active
                    }
                }
            }
        }
    }
}
