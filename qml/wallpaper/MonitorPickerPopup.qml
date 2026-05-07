import Quickshell
import QtQuick
import QtQuick.Effects
import QtQuick.Shapes
import ".."
import "../services"

Rectangle {
  id: monitorPicker
  visible: false
  anchors.fill: parent
  z: 200
  activeFocusOnTab: true
  color: monitorPicker.colors ? Qt.rgba(monitorPicker.colors.surface.r, monitorPicker.colors.surface.g, monitorPicker.colors.surface.b, 0.97)
                               : Qt.rgba(0.08, 0.08, 0.12, 0.97)
  radius: 8

  property var _restoreFocusTo: null

  property var colors
  property var _pendingItem: null
  property var _selectedOutputs: []
  property var _outputState: ({})
  property var wallpaperService: null
  signal accepted(var item, var outputs, var audioMap, var volumeMap)
  signal cancelled()

  readonly property bool _hasAudio: _pendingItem
                                    && (_pendingItem.type === "video" || _pendingItem.type === "we")

  function _thumbForEntry(entry) {
    if (!entry || !wallpaperService) return ""
    var model = wallpaperService.filteredModel
    if (!model) return ""
    var matchPath = entry.path || ""
    var matchWeId = entry.we_id || ""
    for (var i = 0; i < model.count; i++) {
      var row = model.get(i)
      if (matchWeId && row.weId === matchWeId) return row.thumb || ""
      if (matchPath && row.path === matchPath) return row.thumb || ""
    }
    return ""
  }

  function open(item) {
    _pendingItem = item
    var win = monitorPicker.Window ? monitorPicker.Window.window : null
    monitorPicker._restoreFocusTo = win ? win.activeFocusItem : null
    var screens = Quickshell.screens
    var defaultAudio = !Config.wallpaperMute
    var defaultVolume = Config.wallpaperVolume
    DaemonClient.outputs(function(result, error) {
      var state = (!error && result && result.outputs) ? result.outputs : {}
      monitorPicker._outputState = state
      var initial = []
      var fallback = state["*"] || null
      for (var i = 0; i < screens.length; i++) {
        var entry = state[screens[i].name] || fallback || null
        var thumb = monitorPicker._thumbForEntry(entry)
        initial.push({
          name: screens[i].name,
          width: screens[i].width,
          height: screens[i].height,
          selected: false,
          audio: defaultAudio,
          volume: defaultVolume,
          currentThumb: thumb
        })
      }
      monitorPicker._selectedOutputs = initial
      monitorPicker.visible = true
      monitorPicker.forceActiveFocus()
    })
  }

  function close() {
    visible = false
    _pendingItem = null
    _selectedOutputs = []
    _outputState = ({})
    if (_restoreFocusTo) {
      _restoreFocusTo.forceActiveFocus()
      _restoreFocusTo = null
    }
  }

  MouseArea {
    anchors.fill: parent
    acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
    onClicked: function(mouse) { mouse.accepted = true }
    onPressed: function(mouse) { mouse.accepted = true }
    onWheel: function(wheel) { wheel.accepted = true }
  }

  Column {
    anchors.centerIn: parent
    spacing: 12
    width: parent.width * 0.7

    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: "\u{f0379}"
      font.family: Style.fontFamilyNerdIcons; font.pixelSize: 28
      color: monitorPicker.colors ? monitorPicker.colors.primary : "#7986cb"
    }

    Text {
      anchors.horizontalCenter: parent.horizontalCenter
      text: "SELECT MONITORS"
      font.family: Style.fontFamily; font.pixelSize: 14; font.weight: Font.Bold; font.letterSpacing: 1.5
      color: monitorPicker.colors ? monitorPicker.colors.surfaceText : "#fff"
    }

    Text {
      width: parent.width
      horizontalAlignment: Text.AlignHCenter
      text: "Choose which monitors to apply the wallpaper to."
      font.family: Style.fontFamily; font.pixelSize: 11; font.letterSpacing: 0.2
      color: monitorPicker.colors ? Qt.rgba(monitorPicker.colors.surfaceText.r, monitorPicker.colors.surfaceText.g, monitorPicker.colors.surfaceText.b, 0.6)
                                  : Qt.rgba(1, 1, 1, 0.5)
      wrapMode: Text.WordWrap
      lineHeight: 1.3
    }

    Item { width: 1; height: 4 }

    Column {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: 6

      Repeater {
        model: monitorPicker._selectedOutputs.length

        Rectangle {
          id: rowRect
          width: 320
          height: {
            var e = monitorPicker._selectedOutputs[index]
            return (monitorPicker._hasAudio && e && e.selected && e.audio) ? 96 : 56
          }
          radius: 6
          color: {
            var entry = monitorPicker._selectedOutputs[index]
            if (!entry) return Qt.rgba(0, 0, 0, 0.2)
            return entry.selected
              ? (monitorPicker.colors ? Qt.rgba(monitorPicker.colors.primary.r, monitorPicker.colors.primary.g, monitorPicker.colors.primary.b, 0.18) : Qt.rgba(0.3, 0.4, 0.7, 0.18))
              : Qt.rgba(0, 0, 0, 0.15)
          }
          border.width: 1
          border.color: {
            var entry = monitorPicker._selectedOutputs[index]
            if (!entry) return "transparent"
            return entry.selected
              ? (monitorPicker.colors ? Qt.rgba(monitorPicker.colors.primary.r, monitorPicker.colors.primary.g, monitorPicker.colors.primary.b, 0.4) : Qt.rgba(0.3, 0.4, 0.7, 0.4))
              : "transparent"
          }
          anchors.horizontalCenter: parent.horizontalCenter

          Behavior on height { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }

          Item {
            id: topRow
            height: 56
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: 10; anchors.rightMargin: 10

            Rectangle {
              id: monCheckbox
              anchors.verticalCenter: parent.verticalCenter
              anchors.left: parent.left
              width: 16; height: 16; radius: 3
              color: {
                var entry = monitorPicker._selectedOutputs[index]
                if (!entry) return "transparent"
                return entry.selected
                  ? (monitorPicker.colors ? monitorPicker.colors.primary : "#7986cb")
                  : "transparent"
              }
              border.width: 1
              border.color: {
                var entry = monitorPicker._selectedOutputs[index]
                if (!entry) return Qt.rgba(1, 1, 1, 0.3)
                return entry.selected
                  ? (monitorPicker.colors ? monitorPicker.colors.primary : "#7986cb")
                  : Qt.rgba(1, 1, 1, 0.3)
              }

              Text {
                anchors.centerIn: parent
                text: "\u2713"
                font.pixelSize: 11; font.weight: Font.Bold
                color: "#fff"
                visible: {
                  var entry = monitorPicker._selectedOutputs[index]
                  return entry ? entry.selected : false
                }
              }
            }

            Item {
              id: thumbHolder
              anchors.verticalCenter: parent.verticalCenter
              anchors.left: monCheckbox.right
              anchors.leftMargin: 10
              width: 64; height: 36
              clip: true
              opacity: {
                var entry = monitorPicker._selectedOutputs[index]
                return entry && entry.selected ? 1.0 : 0.4
              }

              Rectangle {
                anchors.fill: parent
                radius: 3
                color: Qt.rgba(0, 0, 0, 0.3)
                visible: !thumbImage.visible
              }

              Image {
                id: thumbImage
                anchors.fill: parent
                source: {
                  var entry = monitorPicker._selectedOutputs[index]
                  return entry ? (entry.currentThumb || "") : ""
                }
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
                cache: true
                visible: status === Image.Ready
                layer.enabled: true
                layer.effect: MultiEffect {
                    maskEnabled: true
                    maskSource: thumbMask
                }
              }

              Item {
                id: thumbMask
                anchors.fill: parent
                layer.enabled: true
                visible: false
                Rectangle { anchors.fill: parent; radius: 3; color: "white" }
              }
            }

            Text {
              anchors.verticalCenter: parent.verticalCenter
              anchors.left: thumbHolder.right; anchors.leftMargin: 10
              text: {
                var entry = monitorPicker._selectedOutputs[index]
                return entry ? entry.name : ""
              }
              font.family: Style.fontFamily; font.pixelSize: 12; font.weight: Font.DemiBold
              color: monitorPicker.colors ? monitorPicker.colors.surfaceText : "#fff"
            }

            Text {
              id: resoText
              anchors.verticalCenter: parent.verticalCenter
              anchors.right: parent.right
              anchors.rightMargin: monitorPicker._hasAudio ? 38 : 0
              text: {
                var entry = monitorPicker._selectedOutputs[index]
                return entry ? entry.width + "×" + entry.height : ""
              }
              font.family: Style.fontFamily; font.pixelSize: 10
              color: monitorPicker.colors ? Qt.rgba(monitorPicker.colors.surfaceText.r, monitorPicker.colors.surfaceText.g, monitorPicker.colors.surfaceText.b, 0.5) : Qt.rgba(1, 1, 1, 0.4)
            }
          }

          MouseArea {
            anchors.fill: topRow
            cursorShape: Qt.PointingHandCursor
            onClicked: {
              var arr = monitorPicker._selectedOutputs.slice()
              var e = arr[index]
              arr[index] = {
                name: e.name, width: e.width, height: e.height,
                currentThumb: e.currentThumb,
                selected: !e.selected, audio: e.audio,
                volume: e.volume
              }
              monitorPicker._selectedOutputs = arr
            }
          }

          Rectangle {
            id: audioBtn
            visible: monitorPicker._hasAudio
            anchors.verticalCenter: topRow.verticalCenter
            anchors.right: parent.right
            anchors.rightMargin: 10
            width: 28; height: 28; radius: 4
            z: 1
            color: {
              var entry = monitorPicker._selectedOutputs[index]
              if (!entry || !entry.audio) return Qt.rgba(0, 0, 0, 0.25)
              return monitorPicker.colors ? Qt.rgba(monitorPicker.colors.primary.r, monitorPicker.colors.primary.g, monitorPicker.colors.primary.b, 0.35) : Qt.rgba(0.3, 0.4, 0.7, 0.35)
            }
            border.width: 1
            border.color: {
              var entry = monitorPicker._selectedOutputs[index]
              if (!entry || !entry.audio) return Qt.rgba(1, 1, 1, 0.15)
              return monitorPicker.colors ? monitorPicker.colors.primary : "#7986cb"
            }

            Text {
              anchors.centerIn: parent
              text: {
                var entry = monitorPicker._selectedOutputs[index]
                return (entry && entry.audio) ? "\u{f057e}" : "\u{f0581}"
              }
              font.family: Style.fontFamilyNerdIcons
              font.pixelSize: 14
              color: {
                var entry = monitorPicker._selectedOutputs[index]
                if (!entry || !entry.audio) return Qt.rgba(1, 1, 1, 0.5)
                return monitorPicker.colors ? monitorPicker.colors.surfaceText : "#fff"
              }
            }

            MouseArea {
              anchors.fill: parent
              cursorShape: Qt.PointingHandCursor
              onClicked: function(mouse) {
                mouse.accepted = true
                var arr = monitorPicker._selectedOutputs.slice()
                var e = arr[index]
                arr[index] = {
                  name: e.name, width: e.width, height: e.height,
                  currentThumb: e.currentThumb,
                  selected: e.selected, audio: !e.audio,
                  volume: e.volume
                }
                monitorPicker._selectedOutputs = arr
              }
            }
          }

          Item {
            id: volSliderHolder
            visible: {
              var e = monitorPicker._selectedOutputs[index]
              return monitorPicker._hasAudio && e && e.selected && e.audio
            }
            anchors.top: topRow.bottom
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.leftMargin: 16; anchors.rightMargin: 16
            anchors.bottom: parent.bottom
            anchors.bottomMargin: 8
            z: 2

            readonly property int _value: {
              var e = monitorPicker._selectedOutputs[index]
              return (e && typeof e.volume === "number") ? e.volume : 80
            }
            readonly property real _ratio: Math.max(0, Math.min(1, _value / 100))
            readonly property real _skew: 4
            readonly property real _fillW: Math.round(_track.width * _ratio)

            function _commit(v) {
              var arr = monitorPicker._selectedOutputs.slice()
              var e = arr[index]
              arr[index] = {
                name: e.name, width: e.width, height: e.height,
                currentThumb: e.currentThumb,
                selected: e.selected, audio: e.audio,
                volume: v
              }
              monitorPicker._selectedOutputs = arr
            }

            Item {
              id: _track
              anchors.left: parent.left
              anchors.right: _valTxt.left
              anchors.rightMargin: 8
              anchors.verticalCenter: parent.verticalCenter
              height: 16

              Shape {
                anchors.fill: parent
                layer.enabled: true
                layer.smooth: true
                layer.samples: 4
                preferredRendererType: Shape.CurveRenderer

                ShapePath {
                  fillColor: monitorPicker.colors
                    ? Qt.rgba(monitorPicker.colors.surfaceVariant.r, monitorPicker.colors.surfaceVariant.g, monitorPicker.colors.surfaceVariant.b, 0.6)
                    : Qt.rgba(0.3, 0.3, 0.35, 0.6)
                  strokeColor: monitorPicker.colors
                    ? Qt.rgba(monitorPicker.colors.outline.r, monitorPicker.colors.outline.g, monitorPicker.colors.outline.b, 0.3)
                    : Qt.rgba(1, 1, 1, 0.1)
                  strokeWidth: 1
                  startX: volSliderHolder._skew; startY: 0
                  PathLine { x: _track.width;                       y: 0 }
                  PathLine { x: _track.width - volSliderHolder._skew; y: _track.height }
                  PathLine { x: 0;                                  y: _track.height }
                  PathLine { x: volSliderHolder._skew;             y: 0 }
                }
              }

              Shape {
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                width: Math.max(volSliderHolder._skew * 2 + 1, volSliderHolder._fillW)
                visible: volSliderHolder._ratio > 0
                layer.enabled: true
                layer.smooth: true
                layer.samples: 4
                preferredRendererType: Shape.CurveRenderer

                ShapePath {
                  fillColor: monitorPicker.colors ? monitorPicker.colors.primary : "#7986cb"
                  strokeWidth: 0
                  startX: volSliderHolder._skew; startY: 0
                  PathLine { x: parent.width;                              y: 0 }
                  PathLine { x: parent.width - volSliderHolder._skew;     y: _track.height }
                  PathLine { x: 0;                                         y: _track.height }
                  PathLine { x: volSliderHolder._skew;                    y: 0 }
                }
              }

              Item {
                id: _thumb
                width: 8
                height: _track.height + 8
                anchors.verticalCenter: parent.verticalCenter
                x: Math.max(0, Math.min(_track.width - width, volSliderHolder._fillW - width / 2))

                Shape {
                  anchors.fill: parent
                  layer.enabled: true
                  layer.smooth: true
                  layer.samples: 4
                  preferredRendererType: Shape.CurveRenderer

                  ShapePath {
                    fillColor: monitorPicker.colors ? monitorPicker.colors.primaryText : "#000"
                    strokeColor: monitorPicker.colors ? monitorPicker.colors.primary : "#7986cb"
                    strokeWidth: 1
                    startX: 2.5; startY: 0
                    PathLine { x: _thumb.width;             y: 0 }
                    PathLine { x: _thumb.width - 2.5;       y: _thumb.height }
                    PathLine { x: 0;                         y: _thumb.height }
                    PathLine { x: 2.5;                       y: 0 }
                  }
                }
              }

              MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                preventStealing: true
                function _setFromX(x) {
                  var w = _track.width
                  if (w <= 0) return
                  var clamped = Math.max(0, Math.min(w, x))
                  volSliderHolder._commit(Math.round((clamped / w) * 100))
                }
                onPressed: function(mouse) { mouse.accepted = true; _setFromX(mouse.x) }
                onPositionChanged: function(mouse) { if (pressed) _setFromX(mouse.x) }
                onWheel: function(ev) {
                  ev.accepted = true
                  var step = ev.angleDelta.y > 0 ? 2 : -2
                  var v = Math.max(0, Math.min(100, volSliderHolder._value + step))
                  volSliderHolder._commit(v)
                }
              }
            }

            Text {
              id: _valTxt
              anchors.verticalCenter: parent.verticalCenter
              anchors.right: parent.right
              text: volSliderHolder._value + "%"
              width: 30
              horizontalAlignment: Text.AlignRight
              font.family: Style.fontFamilyCode
              font.pixelSize: 10
              font.weight: Font.Bold
              color: monitorPicker.colors ? monitorPicker.colors.primary : Qt.rgba(1, 1, 1, 0.7)
            }
          }
        }
      }
    }

    Item { width: 1; height: 2 }

    Row {
      anchors.horizontalCenter: parent.horizontalCenter
      spacing: 8

      FilterButton {
        colors: monitorPicker.colors
        label: "CANCEL"
        skew: 8; height: 26
        onClicked: { monitorPicker.cancelled(); monitorPicker.close() }
      }

      FilterButton {
        id: _acceptBtn
        property bool canAccept: {
          for (var i = 0; i < monitorPicker._selectedOutputs.length; i++)
            if (monitorPicker._selectedOutputs[i].selected) return true
          return false
        }
        colors: monitorPicker.colors
        label: "ACCEPT"
        skew: 8; height: 26
        hasActiveColor: true
        activeColor: canAccept ? (monitorPicker.colors ? monitorPicker.colors.primary : "#7986cb") : Qt.rgba(0.5, 0.5, 0.5, 0.3)
        isActive: canAccept
        activeOpacity: canAccept ? 1.0 : 0.4
        onClicked: {
          if (!canAccept) return
          var selected = []
          var audioMap = monitorPicker._hasAudio ? ({}) : null
          var volumeMap = monitorPicker._hasAudio ? ({}) : null
          for (var i = 0; i < monitorPicker._selectedOutputs.length; i++) {
            var e = monitorPicker._selectedOutputs[i]
            if (!e.selected) continue
            selected.push(e.name)
            if (audioMap) audioMap[e.name] = !e.audio
            if (volumeMap) volumeMap[e.name] = (typeof e.volume === "number") ? e.volume : 80
          }
          monitorPicker.accepted(monitorPicker._pendingItem, selected, audioMap, volumeMap)
          monitorPicker.close()
        }
      }
    }
  }

  Keys.onEscapePressed: function(event) {
    event.accepted = true
    monitorPicker.cancelled()
    monitorPicker.close()
  }
}
