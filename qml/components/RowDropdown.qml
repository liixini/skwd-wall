import QtQuick
import QtQuick.Window
import QtQuick.Shapes
import ".."

SettingsRow {
  id: row
  property var model: []
  property string value: ""
  property var onSelect
  property bool _open: false

  readonly property string _currentLabel: {
    for (var i = 0; i < model.length; i++) {
      if (model[i].mode === value) return model[i].label
    }
    return value
  }

  Item {
    id: trigger
    width: 220 * Config.uiScale
    height: 28 * Config.uiScale
    readonly property int _ch: 5

    Shape {
      id: trigShape
      anchors.fill: parent
      antialiasing: true
      preferredRendererType: Shape.CurveRenderer
      ShapePath {
        fillColor: row.colors ? Qt.rgba(row.colors.surfaceContainer.r, row.colors.surfaceContainer.g, row.colors.surfaceContainer.b, 0.92) : Qt.rgba(0.15, 0.15, 0.2, 0.92)
        strokeColor: row._open
          ? (row.colors ? row.colors.primary : Qt.rgba(0.5, 0.7, 1.0, 1.0))
          : (row.colors ? Qt.rgba(row.colors.outline.r, row.colors.outline.g, row.colors.outline.b, 0.28) : Qt.rgba(1, 1, 1, 0.14))
        strokeWidth: row._open ? 2 : 1
        Behavior on strokeColor { ColorAnimation { duration: 160 } }
        startX: trigger._ch; startY: 0
        PathLine { x: trigShape.width;               y: 0 }
        PathLine { x: trigShape.width;               y: trigShape.height - trigger._ch }
        PathLine { x: trigShape.width - trigger._ch; y: trigShape.height }
        PathLine { x: 0;                              y: trigShape.height }
        PathLine { x: 0;                              y: trigger._ch }
        PathLine { x: trigger._ch;                    y: 0 }
      }
    }

    Text {
      anchors.left: parent.left
      anchors.leftMargin: 10 * Config.uiScale
      anchors.verticalCenter: parent.verticalCenter
      anchors.right: chevron.left
      anchors.rightMargin: 8 * Config.uiScale
      text: row._currentLabel
      elide: Text.ElideRight
      font.family: Style.fontFamily
      font.pixelSize: 12 * Config.uiScale
      color: row.colors ? row.colors.surfaceText : "#ffffff"
    }

    Text {
      id: chevron
      anchors.right: parent.right
      anchors.rightMargin: 10 * Config.uiScale
      anchors.verticalCenter: parent.verticalCenter
      text: "▾"
      font.pixelSize: 11 * Config.uiScale
      color: row._open
        ? (row.colors ? row.colors.primary : Qt.rgba(0.5, 0.7, 1.0, 1.0))
        : (row.colors ? Qt.rgba(row.colors.surfaceText.r, row.colors.surfaceText.g, row.colors.surfaceText.b, 0.55) : Qt.rgba(1, 1, 1, 0.4))
      rotation: row._open ? 180 : 0
      Behavior on rotation { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
      Behavior on color    { ColorAnimation  { duration: 160 } }
    }

    MouseArea {
      anchors.fill: parent
      cursorShape: Qt.PointingHandCursor
      onClicked: function(mouse) { mouse.accepted = true; row._open = !row._open }
    }
  }

  Item {
    id: backdrop
    parent: row.Window.contentItem
    visible: row._open
    anchors.fill: parent
    z: 9998
    MouseArea {
      anchors.fill: parent
      acceptedButtons: Qt.LeftButton | Qt.RightButton
      onClicked: row._open = false
    }
  }

  Item {
    id: popup
    parent: row.Window.contentItem
    visible: row._open
    z: 9999
    width: trigger.width
    height: optionsCol.implicitHeight + 8 * Config.uiScale
    readonly property int _ch: 7

    property real _belowY: 0
    property real _aboveY: 0
    property bool _flipUp: false

    function _sync() {
      if (!parent) return
      var below = trigger.mapToItem(parent, 0, trigger.height + 4)
      var above = trigger.mapToItem(parent, 0, -popup.height - 4)
      popup._belowY = below.y
      popup._aboveY = above.y
      popup._flipUp = (below.y + popup.height) > parent.height - 8
      popup.x = below.x
      popup.y = popup._flipUp ? popup._aboveY : popup._belowY
    }

    Timer {
      interval: 33
      running: row._open
      repeat: true
      onTriggered: popup._sync()
    }

    onVisibleChanged: if (visible) popup._sync()

    Shape {
      id: popShape
      anchors.fill: parent
      antialiasing: true
      preferredRendererType: Shape.CurveRenderer
      ShapePath {
        fillColor: row.colors ? Qt.rgba(row.colors.surfaceContainer.r, row.colors.surfaceContainer.g, row.colors.surfaceContainer.b, 0.97) : Qt.rgba(0.10, 0.12, 0.18, 0.97)
        strokeColor: row.colors ? Qt.rgba(row.colors.outline.r, row.colors.outline.g, row.colors.outline.b, 0.32) : Qt.rgba(1, 1, 1, 0.16)
        strokeWidth: 1
        startX: popup._ch; startY: 0
        PathLine { x: popShape.width;             y: 0 }
        PathLine { x: popShape.width;             y: popShape.height - popup._ch }
        PathLine { x: popShape.width - popup._ch; y: popShape.height }
        PathLine { x: 0;                          y: popShape.height }
        PathLine { x: 0;                          y: popup._ch }
        PathLine { x: popup._ch;                  y: 0 }
      }
    }

    Column {
      id: optionsCol
      anchors.top: parent.top
      anchors.topMargin: 4 * Config.uiScale
      width: parent.width

      Repeater {
        model: row.model
        delegate: Item {
          required property var modelData
          width: optionsCol.width
          height: 24 * Config.uiScale
          property bool _isActive: modelData.mode === row.value

          Rectangle {
            anchors.fill: parent
            color: optMouse.containsMouse
              ? (row.colors ? Qt.rgba(row.colors.primary.r, row.colors.primary.g, row.colors.primary.b, 0.18) : Qt.rgba(1, 1, 1, 0.08))
              : "transparent"
          }

          Text {
            anchors.left: parent.left
            anchors.leftMargin: 10 * Config.uiScale
            anchors.verticalCenter: parent.verticalCenter
            text: parent.modelData.label
            font.family: Style.fontFamily
            font.pixelSize: 12 * Config.uiScale
            font.weight: parent._isActive ? Font.Bold : Font.Normal
            color: parent._isActive
              ? (row.colors ? row.colors.primary : "#7986cb")
              : (row.colors ? row.colors.surfaceText : "#ffffff")
          }

          MouseArea {
            id: optMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: function(mouse) {
              mouse.accepted = true
              if (row.onSelect) row.onSelect(parent.modelData.mode)
              row._open = false
            }
          }
        }
      }
    }
  }
}
