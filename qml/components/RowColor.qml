import QtQuick
import QtQuick.Window
import ".."

SettingsRow {
    id: row
    property color value: "#000000"
    property var onCommit
    property bool _open: false

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
        width: 56 * Config.uiScale
        height: 26 * Config.uiScale
        radius: 5
        color: row.value
        border.width: row._open ? 2 : 1
        border.color: row._open
            ? (row.colors ? row.colors.primary : Qt.rgba(0.5, 0.7, 1.0, 1.0))
            : (row.colors ? Qt.rgba(row.colors.outline.r, row.colors.outline.g, row.colors.outline.b, 0.4) : Qt.rgba(1, 1, 1, 0.3))
        Behavior on border.color { ColorAnimation { duration: 160 } }

        Text {
            anchors.centerIn: parent
            text: row._hex(row.value)
            font.family: Style.fontFamilyCode
            font.pixelSize: 10 * Config.uiScale
            color: {
                var c = row.value
                var lum = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
                return lum > 0.55 ? "#000000" : "#ffffff"
            }
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: function(mouse) { mouse.accepted = true; row._open = !row._open }
        }
    }

    Item {
        id: backdrop
        parent: row.Window.contentItem
        visible: row._open
        anchors.fill: parent
        z: 9998
        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            onClicked: row._open = false
        }
    }

    Rectangle {
        id: popup
        parent: row.Window.contentItem
        visible: row._open
        z: 9999
        width: 230
        height: 260
        radius: 8
        color: row.colors
            ? Qt.rgba(row.colors.surface.r, row.colors.surface.g, row.colors.surface.b, 0.98)
            : Qt.rgba(0.08, 0.08, 0.12, 0.98)
        border.width: 1
        border.color: row.colors
            ? Qt.rgba(row.colors.primary.r, row.colors.primary.g, row.colors.primary.b, 0.2)
            : Qt.rgba(1, 1, 1, 0.1)

        function _sync() {
            if (!parent) return
            var below = swatch.mapToItem(parent, swatch.width - popup.width, swatch.height + 6)
            var above = swatch.mapToItem(parent, swatch.width - popup.width, -popup.height - 6)
            var flipUp = (below.y + popup.height) > parent.height - 8
            popup.x = Math.max(8, below.x)
            popup.y = flipUp ? above.y : below.y
        }

        Timer {
            interval: 33
            running: row._open
            repeat: true
            onTriggered: popup._sync()
        }

        onVisibleChanged: if (visible) popup._sync()

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
