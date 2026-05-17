import QtQuick
import QtQuick.Shapes
import ".."

SettingsRow {
  id: row
  property bool checked: false
  property var onToggle

  onClicked: if (onToggle) onToggle(!checked)

  Item {
    id: track
    width: 40 * Config.uiScale
    height: 20 * Config.uiScale

    property real _skew: 4

    Shape {
      anchors.fill: parent
      antialiasing: true
      preferredRendererType: Shape.CurveRenderer
      layer.enabled: true
      layer.samples: 8
      layer.smooth: true
      ShapePath {
        fillColor: row.checked
          ? (row.colors ? row.colors.primary : Style.fallbackAccent)
          : (row.colors ? Qt.rgba(row.colors.surfaceVariant.r, row.colors.surfaceVariant.g, row.colors.surfaceVariant.b, 0.6) : Qt.rgba(0.3, 0.3, 0.35, 0.6))
        strokeColor: row.checked
          ? (row.colors ? Qt.rgba(row.colors.primary.r, row.colors.primary.g, row.colors.primary.b, 0.8) : Style.fallbackAccent)
          : (row.colors ? Qt.rgba(row.colors.outline.r, row.colors.outline.g, row.colors.outline.b, 0.3) : Qt.rgba(1, 1, 1, 0.1))
        strokeWidth: 1
        startX: track._skew; startY: 0
        PathLine { x: track.width;              y: 0 }
        PathLine { x: track.width - track._skew; y: track.height }
        PathLine { x: 0;                         y: track.height }
        PathLine { x: track._skew;               y: 0 }
      }
    }

    Item {
      id: thumb
      width: 16 * Config.uiScale
      height: 14 * Config.uiScale
      anchors.verticalCenter: parent.verticalCenter
      x: row.checked ? parent.width - width - 3 : 3
      Behavior on x { NumberAnimation { duration: Style.animFast; easing.type: Easing.OutCubic } }

      Shape {
        anchors.fill: parent
        antialiasing: true
        preferredRendererType: Shape.CurveRenderer
        layer.enabled: true
        layer.samples: 8
        layer.smooth: true
        ShapePath {
          fillColor: row.checked
            ? (row.colors ? row.colors.primaryText : "#000")
            : (row.colors ? Qt.rgba(row.colors.surfaceText.r, row.colors.surfaceText.g, row.colors.surfaceText.b, 0.7) : Qt.rgba(1, 1, 1, 0.5))
          strokeWidth: 0
          startX: track._skew * 0.6; startY: 0
          PathLine { x: thumb.width;                    y: 0 }
          PathLine { x: thumb.width - track._skew * 0.6; y: thumb.height }
          PathLine { x: 0;                               y: thumb.height }
          PathLine { x: track._skew * 0.6;              y: 0 }
        }
      }
    }
  }
}
