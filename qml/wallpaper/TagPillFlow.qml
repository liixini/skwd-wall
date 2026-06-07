import QtQuick
import ".."
import "../services"

Item {
    id: root

    property var colors
    property var tags: []
    property bool retagging: false
    property int pillHeight: 26
    property int pillFontSize: 11
    property int pillSpacing: 6
    property int pillPadding: 28

    signal removeRequested(string tag)
    signal transitionDone()

    readonly property real contentHeight: _flow.implicitHeight

    property var _displayed: []
    property bool tagsExiting: false

    function _eq(a, b) {
        if (!a || !b || a.length !== b.length) return false
        for (var i = 0; i < a.length; i++) if (a[i] !== b[i]) return false
        return true
    }

    Component.onCompleted: _displayed = tags
    onTagsChanged: _sync()

    function _sync() {
        if (_eq(_displayed, tags)) { if (retagging) transitionDone(); return }
        if (!retagging || _displayed.length === 0) { _displayed = tags; return }
        tagsExiting = true
        _swapTimer.restart()
    }

    Timer {
        id: _swapTimer
        interval: 280
        onTriggered: {
            root._displayed = root.tags
            root.tagsExiting = false
            root.transitionDone()
        }
    }

    Flickable {
        anchors.fill: parent
        contentHeight: _flow.implicitHeight
        clip: true
        flickableDirection: Flickable.VerticalFlick
        boundsBehavior: Flickable.StopAtBounds

        Flow {
            id: _flow
            width: parent.width
            spacing: root.pillSpacing

            Repeater {
                model: root._displayed

                Rectangle {
                    id: pill
                    required property int index
                    required property var modelData
                    property bool hovered: _ma.containsMouse
                    width: _txt.implicitWidth + root.pillPadding
                    height: root.pillHeight
                    radius: 4
                    color: hovered
                        ? (root.colors ? Qt.rgba(root.colors.surfaceVariant.r, root.colors.surfaceVariant.g, root.colors.surfaceVariant.b, 0.5) : Qt.rgba(1, 1, 1, 0.15))
                        : "transparent"
                    border.width: 1
                    border.color: hovered
                        ? (root.colors ? Qt.rgba(root.colors.primary.r, root.colors.primary.g, root.colors.primary.b, 0.7) : Qt.rgba(1, 1, 1, 0.3))
                        : (root.colors ? Qt.rgba(root.colors.outline.r, root.colors.outline.g, root.colors.outline.b, 0.5) : Qt.rgba(1, 1, 1, 0.15))
                    Behavior on color { ColorAnimation { duration: Style.animVeryFast } }
                    Behavior on border.color { ColorAnimation { duration: Style.animVeryFast } }

                    transform: Matrix4x4 {
                        matrix: Qt.matrix4x4(
                            1, -0.08, 0, 0,
                            0, 1,     0, 0,
                            0, 0,     1, 0,
                            0, 0,     0, 1)
                    }

                    opacity: 0
                    scale: 0.85
                    Component.onCompleted: _enter.start()

                    SequentialAnimation {
                        id: _enter
                        PauseAnimation { duration: Math.min(pill.index * 25, 600) }
                        ParallelAnimation {
                            NumberAnimation { target: pill; property: "opacity"; to: 1; duration: 220; easing.type: Easing.OutCubic }
                            NumberAnimation { target: pill; property: "scale"; to: 1.0; duration: 220; easing.type: Easing.OutBack; easing.overshoot: 1.4 }
                        }
                    }

                    SequentialAnimation {
                        id: _exit
                        PauseAnimation { duration: Math.min(pill.index * 14, 130) }
                        ParallelAnimation {
                            NumberAnimation { target: pill; property: "opacity"; to: 0; duration: 150; easing.type: Easing.InCubic }
                            NumberAnimation { target: pill; property: "scale"; to: 0.8; duration: 150; easing.type: Easing.InCubic }
                        }
                    }

                    Connections {
                        target: root
                        function onTagsExitingChanged() {
                            if (root.tagsExiting) _exit.start()
                        }
                    }

                    Text {
                        id: _txt
                        anchors.left: parent.left; anchors.leftMargin: 8
                        anchors.verticalCenter: parent.verticalCenter
                        text: pill.modelData.toUpperCase()
                        color: root.colors ? root.colors.tertiary : "#8bceff"
                        font.family: Style.fontFamily; font.pixelSize: root.pillFontSize
                        font.weight: Font.Medium; font.letterSpacing: 0.5
                    }

                    Text {
                        anchors.right: parent.right; anchors.rightMargin: 6
                        anchors.verticalCenter: parent.verticalCenter
                        text: "\u{f0156}"
                        font.family: Style.fontFamilyNerdIcons; font.pixelSize: 10
                        color: pill.hovered ? (root.colors ? root.colors.primary : "#ff6b6b") : Qt.rgba(1, 1, 1, 0.25)
                        Behavior on color { ColorAnimation { duration: Style.animVeryFast } }
                    }

                    MouseArea {
                        id: _ma
                        anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                        onClicked: root.removeRequested(pill.modelData)
                    }
                }
            }
        }
    }
}
