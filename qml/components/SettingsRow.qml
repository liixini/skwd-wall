import QtQuick
import ".."

Item {
  id: root
  property var colors
  property string title: ""
  property string description: ""
  default property alias _control: controlSlot.data
  signal clicked()

  width: parent ? parent.width : 0
  implicitHeight: Math.max(
    textCol.implicitHeight + 16 * Config.uiScale,
    (controlSlot.children.length > 0 ? controlSlot.childrenRect.height : 0) + 14 * Config.uiScale
  )

  readonly property bool _isFirst: parent && parent.children.length > 0 && parent.children[0] === root

  Rectangle {
    visible: !root._isFirst
    anchors.top: parent.top
    anchors.left: parent.left
    anchors.right: parent.right
    anchors.leftMargin: 14 * Config.uiScale
    anchors.rightMargin: 14 * Config.uiScale
    height: 1
    gradient: Gradient {
      orientation: Gradient.Horizontal
      GradientStop {
        position: 0.0
        color: root.colors
          ? Qt.rgba(root.colors.primary.r, root.colors.primary.g, root.colors.primary.b, 0.18)
          : Qt.rgba(0.5, 0.7, 1.0, 0.18)
      }
      GradientStop {
        position: 1.0
        color: root.colors
          ? Qt.rgba(root.colors.outline.r, root.colors.outline.g, root.colors.outline.b, 0.06)
          : Qt.rgba(1, 1, 1, 0.04)
      }
    }
  }

  MouseArea {
    id: rowMouse
    anchors.fill: parent
    hoverEnabled: true
    cursorShape: Qt.PointingHandCursor
    onClicked: root.clicked()
  }

  Rectangle {
    id: hoverStripe
    anchors.left: parent.left
    anchors.top: parent.top
    anchors.bottom: parent.bottom
    anchors.topMargin: 4
    anchors.bottomMargin: 4
    width: rowMouse.containsMouse ? 3 : 0
    color: root.colors ? root.colors.primary : Qt.rgba(0.5, 0.7, 1.0, 1.0)
    Behavior on width { NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
  }

  Rectangle {
    anchors.fill: parent
    visible: rowMouse.containsMouse
    color: root.colors
      ? Qt.rgba(root.colors.primary.r, root.colors.primary.g, root.colors.primary.b, 0.05)
      : Qt.rgba(1, 1, 1, 0.025)
  }

  Column {
    id: textCol
    anchors.left: parent.left
    anchors.right: controlSlot.left
    anchors.verticalCenter: parent.verticalCenter
    anchors.leftMargin: 18 * Config.uiScale
    anchors.rightMargin: 12 * Config.uiScale
    spacing: 2 * Config.uiScale

    Text {
      width: parent.width
      text: root.title
      elide: Text.ElideRight
      font.family: Style.fontFamily
      font.pixelSize: 12 * Config.uiScale
      font.weight: Font.Medium
      color: root.colors ? root.colors.surfaceText : "#ffffff"
    }
    Text {
      visible: root.description !== ""
      width: parent.width
      text: root.description
      wrapMode: Text.WordWrap
      font.family: Style.fontFamily
      font.pixelSize: 10 * Config.uiScale
      color: root.colors
        ? Qt.rgba(root.colors.surfaceVariantText.r, root.colors.surfaceVariantText.g, root.colors.surfaceVariantText.b, 0.85)
        : Qt.rgba(1, 1, 1, 0.4)
    }
  }

  Item {
    id: controlSlot
    anchors.right: parent.right
    anchors.verticalCenter: parent.verticalCenter
    anchors.rightMargin: 14 * Config.uiScale
    width: childrenRect.width
    height: childrenRect.height
  }
}
