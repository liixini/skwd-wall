import QtQuick
import QtQuick.Shapes
import ".."

SettingsRow {
  id: row
  property string value: ""
  property string placeholder: ""
  property var onCommit

  Item {
    id: tinBox
    width: 220 * Config.uiScale
    height: 28 * Config.uiScale
    readonly property int _ch: 5

    Shape {
      id: tinShape
      anchors.fill: parent
      antialiasing: true
      preferredRendererType: Shape.CurveRenderer
      ShapePath {
        fillColor: row.colors ? Qt.rgba(row.colors.surfaceContainer.r, row.colors.surfaceContainer.g, row.colors.surfaceContainer.b, 0.92) : Qt.rgba(0.15, 0.15, 0.2, 0.92)
        strokeColor: inputField.activeFocus
          ? (row.colors ? row.colors.primary : Qt.rgba(0.5, 0.7, 1.0, 1.0))
          : (row.colors ? Qt.rgba(row.colors.outline.r, row.colors.outline.g, row.colors.outline.b, 0.28) : Qt.rgba(1, 1, 1, 0.14))
        strokeWidth: inputField.activeFocus ? 2 : 1
        Behavior on strokeColor { ColorAnimation { duration: 120 } }
        startX: tinBox._ch; startY: 0
        PathLine { x: tinShape.width;                y: 0 }
        PathLine { x: tinShape.width;                y: tinShape.height - tinBox._ch }
        PathLine { x: tinShape.width - tinBox._ch;   y: tinShape.height }
        PathLine { x: 0;                             y: tinShape.height }
        PathLine { x: 0;                             y: tinBox._ch }
        PathLine { x: tinBox._ch;                    y: 0 }
      }
    }

    TextInput {
      id: inputField
      anchors.fill: parent
      anchors.leftMargin: 10 * Config.uiScale
      anchors.rightMargin: 10 * Config.uiScale
      verticalAlignment: TextInput.AlignVCenter
      font.family: Style.fontFamily
      font.pixelSize: 12 * Config.uiScale
      color: row.colors ? row.colors.surfaceText : "#ffffff"
      clip: true
      selectByMouse: true
      text: row.value
      onTextEdited: if (row.onCommit) row.onCommit(text)

      Text {
        anchors.fill: parent
        verticalAlignment: Text.AlignVCenter
        text: row.placeholder
        font: parent.font
        color: row.colors ? Qt.rgba(row.colors.surfaceText.r, row.colors.surfaceText.g, row.colors.surfaceText.b, 0.3) : Qt.rgba(1, 1, 1, 0.2)
        visible: !parent.text && !parent.activeFocus
      }
    }
  }
}
