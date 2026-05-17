import QtQuick
import ".."


Item {
  id: root
  property var colors
  property string text: ""

  width: parent ? parent.width : 0
  implicitHeight: 18 * Config.uiScale

  Rectangle {
    id: stripe
    anchors.verticalCenter: parent.verticalCenter
    anchors.left: parent.left
    width: 3
    height: 14 * Config.uiScale
    radius: 1
    color: root.colors ? root.colors.primary : Qt.rgba(0.5, 0.7, 1.0, 1.0)
  }

  Text {
    id: label
    anchors.verticalCenter: parent.verticalCenter
    anchors.left: stripe.right
    anchors.leftMargin: 8
    text: root.text
    font.family: Style.fontFamily
    font.pixelSize: 11 * Config.uiScale
    font.weight: Font.Bold
    font.letterSpacing: 1.4
    color: root.colors ? root.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
  }

  Rectangle {
    anchors.verticalCenter: parent.verticalCenter
    anchors.left: label.right
    anchors.right: parent.right
    anchors.leftMargin: 10
    height: 1
    gradient: Gradient {
      orientation: Gradient.Horizontal
      GradientStop {
        position: 0.0
        color: root.colors
          ? Qt.rgba(root.colors.primary.r, root.colors.primary.g, root.colors.primary.b, 0.35)
          : Qt.rgba(0.5, 0.7, 1.0, 0.35)
      }
      GradientStop {
        position: 1.0
        color: root.colors
          ? Qt.rgba(root.colors.primary.r, root.colors.primary.g, root.colors.primary.b, 0)
          : Qt.rgba(0.5, 0.7, 1.0, 0)
      }
    }
  }
}
