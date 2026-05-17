import QtQuick
import ".."

SettingsRow {
    id: row
    property color value: "#000000"
    property var onCommit

    function _hex(c) {
        function pair(f) {
            var n = Math.round(f * 255)
            var s = n.toString(16)
            return s.length === 1 ? "0" + s : s
        }
        return "#" + pair(c.r) + pair(c.g) + pair(c.b)
    }

    Rectangle {
        id: swatch
        width: 32 * Config.uiScale
        height: 22 * Config.uiScale
        radius: 4
        color: row.value
        border.width: 1
        border.color: row.colors
            ? Qt.rgba(row.colors.outline.r, row.colors.outline.g, row.colors.outline.b, 0.4)
            : Qt.rgba(1, 1, 1, 0.3)

        Text {
            anchors.centerIn: parent
            text: row._hex(row.value)
            font.family: Style.fontFamilyCode
            font.pixelSize: 9 * Config.uiScale
            color: {
                var c = row.value
                var lum = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
                return lum > 0.55 ? "#000000" : "#ffffff"
            }
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: popup.visible = !popup.visible
        }
    }

    Rectangle {
        id: popup
        visible: false
        width: 230
        height: 260
        radius: 8
        z: 1000
        anchors.right: swatch.right
        anchors.top: swatch.bottom
        anchors.topMargin: 6
        color: row.colors
            ? Qt.rgba(row.colors.surface.r, row.colors.surface.g, row.colors.surface.b, 0.98)
            : Qt.rgba(0.08, 0.08, 0.12, 0.98)
        border.width: 1
        border.color: row.colors
            ? Qt.rgba(row.colors.primary.r, row.colors.primary.g, row.colors.primary.b, 0.2)
            : Qt.rgba(1, 1, 1, 0.1)

        MouseArea { anchors.fill: parent; preventStealing: true; onClicked: function(m) { m.accepted = true } }

        ColorWheel {
            anchors.centerIn: parent
            colors: row.colors
            value: row.value
            onCommit: function(v) {
                row.value = v
                if (row.onCommit) row.onCommit(v)
            }
        }
    }
}
