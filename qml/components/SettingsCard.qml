import QtQuick
import QtQuick.Shapes
import QtQuick.Effects
import ".."

Item {
  id: root
  property var colors
  property string title: ""
  property string subtitle: ""
  default property alias _content: contentCol.data
  property alias titleAction: titleActionSlot.data
  property int innerPad: 16
  property int chamfer: 18

  width: parent ? parent.width : 0
  implicitHeight: cardArea.height + 14

  Item {
    id: cardArea
    anchors.left: parent.left
    anchors.right: parent.right
    anchors.top: parent.top
    anchors.leftMargin: 4
    anchors.rightMargin: 4
    height: bodyCol.implicitHeight + root.innerPad * 2

    Shape {
      id: cardShape
      anchors.fill: parent
      antialiasing: true
      preferredRendererType: Shape.CurveRenderer

      ShapePath {
        fillColor: root.colors
          ? Qt.rgba(root.colors.surfaceContainer.r, root.colors.surfaceContainer.g, root.colors.surfaceContainer.b, 0.92)
          : Qt.rgba(0.15, 0.17, 0.22, 0.92)
        strokeColor: "transparent"
        strokeWidth: 0
        startX: root.chamfer
        startY: 0
        PathLine { x: cardShape.width;               y: 0 }
        PathLine { x: cardShape.width;               y: cardShape.height - root.chamfer }
        PathLine { x: cardShape.width - root.chamfer; y: cardShape.height }
        PathLine { x: 0;                              y: cardShape.height }
        PathLine { x: 0;                              y: root.chamfer }
        PathLine { x: root.chamfer;                   y: 0 }
      }

      ShapePath {
        fillColor: "transparent"
        strokeColor: Qt.rgba(1, 1, 1, 0.08)
        strokeWidth: 1
        startX: root.chamfer
        startY: 0
        PathLine { x: cardShape.width;               y: 0 }
      }

      ShapePath {
        fillColor: "transparent"
        strokeColor: root.colors
          ? root.colors.primary
          : Qt.rgba(0.5, 0.7, 1.0, 1.0)
        strokeWidth: 3
        startX: 0
        startY: root.chamfer
        PathLine { x: root.chamfer; y: 0 }
      }
    }

    Column {
      id: bodyCol
      anchors.left: parent.left
      anchors.right: parent.right
      anchors.top: parent.top
      anchors.leftMargin: root.innerPad
      anchors.rightMargin: root.innerPad
      anchors.topMargin: root.innerPad
      spacing: 10

      Item {
        visible: root.title !== "" || root.subtitle !== "" || titleActionSlot.children.length > 0
        width: parent.width
        implicitHeight: Math.max(headerCol.implicitHeight, titleActionSlot.height)

        Column {
          id: headerCol
          anchors.left: parent.left
          anchors.right: titleActionSlot.left
          anchors.verticalCenter: parent.verticalCenter
          anchors.rightMargin: 12
          spacing: 4

          Text {
            visible: root.title !== ""
            text: root.title.toUpperCase()
            elide: Text.ElideRight
            font.family: Style.fontFamily
            font.pixelSize: 10 * Config.uiScale
            font.weight: Font.Bold
            font.letterSpacing: 0.5
            color: root.colors ? root.colors.tertiary : Qt.rgba(0.5, 0.8, 1.0, 1.0)
          }

          Text {
            visible: root.subtitle !== ""
            width: parent.width
            text: root.subtitle
            wrapMode: Text.WordWrap
            font.family: Style.fontFamily
            font.pixelSize: 11
            color: root.colors
              ? Qt.rgba(root.colors.surfaceVariantText.r, root.colors.surfaceVariantText.g, root.colors.surfaceVariantText.b, 0.85)
              : Qt.rgba(1, 1, 1, 0.4)
          }
        }

        Item {
          id: titleActionSlot
          anchors.right: parent.right
          anchors.verticalCenter: parent.verticalCenter
          width: childrenRect.width
          height: childrenRect.height
        }
      }

      Column {
        id: contentCol
        width: parent.width
      }
    }
  }

  layer.enabled: true
  layer.smooth: true
  layer.effect: MultiEffect {
    shadowEnabled: true
    shadowBlur: 0.8
    shadowVerticalOffset: 4
    shadowHorizontalOffset: 0
    shadowColor: Qt.rgba(0, 0, 0, 0.35)
    shadowOpacity: 1.0
    paddingRect: Qt.rect(-10, -3, 20, 16)
  }
}
