import QtQuick
import ".."


Row {
  id: strip

  property var colors
  property int selectedValue: -1
  signal valueSelected(int value)

  readonly property int _skew: 10 * Config.uiScale
  spacing: -_skew

  Repeater {
    model: 13

    Item {
      width: 28 * Config.uiScale
      height: 24 * Config.uiScale
      readonly property int filterValue: index < 12 ? index : 99
      readonly property bool isSelected: strip.selectedValue === filterValue
      readonly property color hueColor: index === 12 ? Qt.hsla(0, 0, 0.45, 1.0) : Qt.hsla(index / 12.0, 0.65, 0.45, 1.0)
      readonly property color hueBright: index === 12 ? Qt.hsla(0, 0, 0.6, 1.0) : Qt.hsla(index / 12.0, 0.75, 0.55, 1.0)
      readonly property bool isHovered: _swatchMouse.containsMouse
      z: isSelected ? 10 : (isHovered ? 5 : 1)

      Canvas {
        anchors.fill: parent
        scale: parent.isSelected ? 1.15 : 1.0
        Behavior on scale { NumberAnimation { duration: Style.animVeryFast; easing.type: Easing.OutBack } }
        property color cFill: parent.isSelected ? parent.hueBright : parent.hueColor
        property color bgCol: strip.colors ? strip.colors.surfaceContainer : Qt.rgba(0.1, 0.12, 0.18, 1.0)
        property bool sel: parent.isSelected
        property bool hov: parent.isHovered
        onCFillChanged: requestPaint()
        onSelChanged: requestPaint()
        onHovChanged: requestPaint()
        onBgColChanged: requestPaint()
        onPaint: {
          var ctx = getContext("2d")
          ctx.clearRect(0, 0, width, height)
          var sk = strip._skew
          ctx.fillStyle = bgCol
          ctx.beginPath()
          ctx.moveTo(sk, 0)
          ctx.lineTo(width, 0)
          ctx.lineTo(width - sk, height)
          ctx.lineTo(0, height)
          ctx.closePath()
          ctx.fill()
          var inset = 1
          var iSk = sk * (height - 2 * inset) / height
          ctx.fillStyle = hov ? Qt.lighter(cFill, 1.2) : cFill
          ctx.beginPath()
          ctx.moveTo(iSk + inset, inset)
          ctx.lineTo(width - inset, inset)
          ctx.lineTo(width - inset - iSk, height - inset)
          ctx.lineTo(inset, height - inset)
          ctx.closePath()
          ctx.fill()
        }
      }

      MouseArea {
        id: _swatchMouse
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        hoverEnabled: true
        onClicked: {
          if (parent.isSelected) strip.valueSelected(-1)
          else strip.valueSelected(parent.filterValue)
        }
      }
    }
  }
}
