import QtQuick
import QtQuick.Controls
import ".."

Item {
    id: btn

    property var colors
    property bool isActive: false
    property string icon: ""
    property string label: ""
    property bool useNerdFont: icon !== ""
    property string tooltip: ""
    property int skew: 10
    property color activeColor: "transparent"
    property bool hasActiveColor: false
    property real activeOpacity: 1.0

    signal clicked()

    width: _label.implicitWidth + 24 * Config.uiScale + skew
    height: 24 * Config.uiScale
    z: isActive ? 10 : (isHovered ? 5 : 1)

    readonly property bool isHovered: _mouse.containsMouse
    readonly property color _resolvedActiveColor: btn.hasActiveColor ? btn.activeColor : (btn.colors ? btn.colors.primary : Style.fallbackAccent)

    Canvas {
        id: _canvas
        anchors.fill: parent

        property color fillColor: btn.isActive
            ? btn._resolvedActiveColor
            : (btn.isHovered
                ? (btn.colors ? Qt.rgba(btn.colors.surfaceVariant.r, btn.colors.surfaceVariant.g, btn.colors.surfaceVariant.b, 0.6) : Qt.rgba(1, 1, 1, 0.15))
                : (btn.colors ? Qt.rgba(btn.colors.surfaceContainer.r, btn.colors.surfaceContainer.g, btn.colors.surfaceContainer.b, 0.85) : Qt.rgba(0.1, 0.12, 0.18, 0.85)))
        property color strokeColor: btn.isActive
            ? Qt.rgba(btn._resolvedActiveColor.r, btn._resolvedActiveColor.g, btn._resolvedActiveColor.b, 0.6)
            : (btn.colors ? Qt.rgba(btn.colors.primary.r, btn.colors.primary.g, btn.colors.primary.b, 0.15) : Qt.rgba(1, 1, 1, 0.08))

        onFillColorChanged: requestPaint()
        onStrokeColorChanged: requestPaint()
        onWidthChanged: requestPaint()

        onPaint: {
            var ctx = getContext("2d")
            ctx.clearRect(0, 0, width, height)
            var sk = btn.skew
            ctx.fillStyle = fillColor
            ctx.beginPath()
            ctx.moveTo(sk, 0)
            ctx.lineTo(width, 0)
            ctx.lineTo(width - sk, height)
            ctx.lineTo(0, height)
            ctx.closePath()
            ctx.fill()
            ctx.strokeStyle = strokeColor
            ctx.lineWidth = 1
            ctx.stroke()
        }
    }

    Text {
        id: _label
        anchors.centerIn: parent
        text: btn.icon || btn.label
        font.pixelSize: (btn.useNerdFont ? 14 : 10) * Config.uiScale
        font.family: btn.useNerdFont ? Style.fontFamilyNerdIcons : Style.fontFamily
        font.weight: btn.useNerdFont ? Font.Normal : Font.Bold
        font.letterSpacing: btn.useNerdFont ? 0 : 0.5
        color: btn.isActive
            ? (btn.hasActiveColor ? "#fff" : (btn.colors ? btn.colors.primaryText : "#000"))
            : (btn.colors ? btn.colors.tertiary : "#8bceff")
    }

    opacity: btn.activeOpacity

    MouseArea {
        id: _mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: btn.clicked()
    }

    StyledToolTip {
        visible: btn.tooltip !== "" && _mouse.containsMouse
        text: btn.tooltip
        delay: Style.tooltipDelay
        colors: btn.colors
    }
}
