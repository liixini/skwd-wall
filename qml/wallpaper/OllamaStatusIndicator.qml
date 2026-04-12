import QtQuick
import ".."

Rectangle {
    id: root

    property var colors
    property bool showing: false
    property bool ollamaActive: false
    property int ollamaTotalThumbs: 0
    property int ollamaTaggedCount: 0
    property string ollamaEta: ""
    property string ollamaLogLine: ""

    visible: ollamaActive
    opacity: ollamaActive ? 1 : 0
    Behavior on opacity { NumberAnimation { duration: Style.animNormal } }

    width: Math.max(statusRow.width + 20, logText.width + 20)
    height: ollamaLogLine ? 44 : 28
    radius: height / 2
    color: colors ? Qt.rgba(colors.surfaceContainer.r, colors.surfaceContainer.g, colors.surfaceContainer.b, 0.9) : Qt.rgba(0.1, 0.12, 0.18, 0.9)

    layer.enabled: false

    Column {
        anchors.centerIn: parent
        spacing: 2

        Row {
            id: statusRow
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: 6

            Text {
                text: "󰔟"
                font.family: Style.fontFamilyNerdIcons
                font.pixelSize: 14
                color: root.colors ? root.colors.primary : "#8BC34A"
                RotationAnimation on rotation {
                    from: 0; to: 360; duration: Style.animSpin
                    loops: Animation.Infinite
                    running: root.ollamaActive && root.showing
                }
            }

            Text {
                text: {
                    if (root.ollamaTaggedCount === 0 && root.ollamaTotalThumbs === 0 && root.ollamaLogLine)
                        return root.ollamaLogLine
                    var status = "ANALYZING"
                    var progress = ""
                    if (root.ollamaTotalThumbs > 0) {
                        progress = " " + root.ollamaTaggedCount + "/" + root.ollamaTotalThumbs
                    }
                    var eta = root.ollamaEta
                    if (eta && eta !== "") return status + progress + " (" + eta + ")"
                    return status + progress
                }
                font.family: Style.fontFamily
                font.pixelSize: 11
                font.weight: Font.Medium
                font.letterSpacing: 0.5
                color: root.colors ? root.colors.tertiary : "#8bceff"
            }
        }

        Text {
            id: logText
            anchors.horizontalCenter: parent.horizontalCenter
            text: root.ollamaLogLine
            visible: root.ollamaLogLine !== ""
            font.family: Style.fontFamilyCode
            font.pixelSize: 9
            color: root.colors ? Qt.rgba(root.colors.surfaceText.r, root.colors.surfaceText.g, root.colors.surfaceText.b, 0.6) : Qt.rgba(1, 1, 1, 0.5)
            elide: Text.ElideMiddle
            maximumLineCount: 1
        }
    }
}
