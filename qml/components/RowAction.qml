import QtQuick
import ".."

SettingsRow {
  id: row
  property string valueLabel: ""

  Row {
    spacing: 8 * Config.uiScale

    Text {
      visible: row.valueLabel !== ""
      text: row.valueLabel
      anchors.verticalCenter: parent.verticalCenter
      font.family: Style.fontFamily
      font.pixelSize: 12 * Config.uiScale
      color: row.colors
        ? Qt.rgba(row.colors.surfaceText.r, row.colors.surfaceText.g, row.colors.surfaceText.b, 0.5)
        : Qt.rgba(1, 1, 1, 0.35)
    }

    Text {
      text: "›"
      anchors.verticalCenter: parent.verticalCenter
      font.family: Style.fontFamily
      font.pixelSize: 16 * Config.uiScale
      color: row.colors
        ? Qt.rgba(row.colors.surfaceText.r, row.colors.surfaceText.g, row.colors.surfaceText.b, 0.5)
        : Qt.rgba(1, 1, 1, 0.35)
    }
  }
}
