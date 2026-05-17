import QtQuick
import QtQuick.Shapes
import ".."

SettingsRow {
  id: row
  property real value: 0
  property real min: 0
  property real max: 1000
  property int decimals: 0
  property string suffix: ""
  property bool enabled: true
  property var onCommit

  opacity: enabled ? 1.0 : 0.45

  Item {
    id: inputBox
    width: 88 * Config.uiScale
    height: 28 * Config.uiScale
    readonly property int _ch: 5

    Shape {
      id: inputShape
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
        startX: inputBox._ch; startY: 0
        PathLine { x: inputShape.width;                  y: 0 }
        PathLine { x: inputShape.width;                  y: inputShape.height - inputBox._ch }
        PathLine { x: inputShape.width - inputBox._ch;   y: inputShape.height }
        PathLine { x: 0;                                 y: inputShape.height }
        PathLine { x: 0;                                 y: inputBox._ch }
        PathLine { x: inputBox._ch;                      y: 0 }
      }
    }

    TextInput {
      id: inputField
      anchors.fill: parent
      anchors.leftMargin: 10 * Config.uiScale
      anchors.rightMargin: 10 * Config.uiScale
      verticalAlignment: TextInput.AlignVCenter
      horizontalAlignment: TextInput.AlignRight
      font.family: Style.fontFamilyCode
      font.pixelSize: 12 * Config.uiScale
      font.weight: Font.Medium
      color: inputField.activeFocus
        ? (row.colors ? row.colors.primary : Qt.rgba(0.5, 0.7, 1.0, 1.0))
        : (row.colors ? row.colors.surfaceText : "#ffffff")
      clip: true
      selectByMouse: true
      enabled: row.enabled
      text: (row.decimals === 0 ? Math.round(row.value).toString() : row.value.toFixed(row.decimals)) + (row.suffix ? " " + row.suffix : "")
      validator: DoubleValidator {
        bottom: row.min
        top: row.max
        decimals: row.decimals
        notation: DoubleValidator.StandardNotation
      }
      onTextEdited: {
        var stripped = row.suffix ? text.replace(row.suffix, "").trim() : text
        var n = parseFloat(stripped)
        if (!isNaN(n) && n >= row.min && n <= row.max && row.onCommit) {
          if (row.decimals === 0) n = Math.round(n)
          row.onCommit(n)
        }
      }
    }
  }
}
