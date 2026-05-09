import QtQuick
import QtQuick.Controls
import ".."
import "../services"

Rectangle {
  id: themePicker
  visible: false
  anchors.fill: parent
  z: 220
  activeFocusOnTab: true
  color: "transparent"

  property var colors
  property var _restoreFocusTo: null
  property var _palettes: ({})
  property string _mode: Config.matugenMode

  readonly property var _schemes: [
    "scheme-fidelity", "scheme-vibrant", "scheme-content", "scheme-expressive",
    "scheme-fruit-salad", "scheme-monochrome", "scheme-neutral", "scheme-rainbow", "scheme-tonal-spot"
  ]
  readonly property int _maxIndices: 4
  property int _activeIndices: 4

  function _recomputeActive() {
    for (var idx = _maxIndices - 1; idx >= 1; idx--) {
      var anyValidAtIdx = false
      for (var s = 0; s < _schemes.length; s++) {
        var e = _palettes[_key(_schemes[s], idx)]
        if (e && !e._missing && e.primary && e.primary.length > 0) { anyValidAtIdx = true; break }
      }
      if (anyValidAtIdx) { _activeIndices = idx + 1; return }
    }
    _activeIndices = 1
  }

  signal applied(string scheme, string mode, int colorIndex)
  signal cancelled()

  function _key(scheme, idx) { return scheme + "|" + idx }

  function open() {
    var win = themePicker.Window ? themePicker.Window.window : null
    themePicker._restoreFocusTo = win ? win.activeFocusItem : null
    themePicker._mode = Config.matugenMode
    themePicker._palettes = ({})
    themePicker.visible = true
    themePicker.forceActiveFocus()
    _refresh()
  }

  function close() {
    visible = false
    _palettes = ({})
    if (_restoreFocusTo) { _restoreFocusTo.forceActiveFocus(); _restoreFocusTo = null }
  }

  function _refresh() {
    var modeAtRequest = _mode
    _activeIndices = _maxIndices
    for (var s = 0; s < _schemes.length; s++) {
      for (var i = 0; i < _maxIndices; i++) {
        (function(scheme, idx) {
          DaemonClient.themePreview(scheme, modeAtRequest, idx, function(result, err) {
            if (themePicker._mode !== modeAtRequest) return
            var copy = {}
            for (var k in themePicker._palettes) copy[k] = themePicker._palettes[k]
            copy[themePicker._key(scheme, idx)] = (err || !result) ? { _missing: true } : result
            themePicker._palettes = copy
            themePicker._recomputeActive()
          })
        })(_schemes[s], i)
      }
    }
  }

  MouseArea {
    anchors.fill: parent
    acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
    onClicked: function(mouse) { mouse.accepted = true }
    onPressed: function(mouse) { mouse.accepted = true }
    onWheel: function(wheel) { wheel.accepted = true }
  }

  Rectangle {
    id: card
    anchors.centerIn: parent
    width: Math.min(parent.width - 48, 640)
    height: Math.min(parent.height - 48, contentCol.implicitHeight + 32)
    radius: 8
    color: themePicker.colors
      ? Qt.rgba(themePicker.colors.surfaceContainer.r, themePicker.colors.surfaceContainer.g, themePicker.colors.surfaceContainer.b, 0.95)
      : Qt.rgba(0.12, 0.12, 0.16, 0.95)
    border.width: 1
    border.color: themePicker.colors ? Qt.rgba(themePicker.colors.outline.r, themePicker.colors.outline.g, themePicker.colors.outline.b, 0.3) : Qt.rgba(1,1,1,0.1)

    MouseArea {
      anchors.fill: parent
      acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
      onClicked: function(mouse) { mouse.accepted = true }
      onPressed: function(mouse) { mouse.accepted = true }
      onWheel: function(wheel) { wheel.accepted = true }
    }

    Column {
      id: contentCol
      anchors.left: parent.left
      anchors.right: parent.right
      anchors.top: parent.top
      anchors.margins: 16
      spacing: 10

      Item {
        width: parent.width
        height: 22

        Text {
          anchors.left: parent.left
          anchors.verticalCenter: parent.verticalCenter
          text: "THEME PICKER"
          font.family: Style.fontFamily; font.pixelSize: 12; font.weight: Font.Bold; font.letterSpacing: 1.4
          color: themePicker.colors ? themePicker.colors.surfaceText : "#fff"
        }

        Row {
          anchors.right: parent.right
          anchors.verticalCenter: parent.verticalCenter
          spacing: 4

          Repeater {
            model: ["dark", "light"]
            Rectangle {
              required property string modelData
              width: 42; height: 18; radius: 3
              color: themePicker._mode === modelData
                ? (themePicker.colors ? themePicker.colors.primary : "#7986cb")
                : Qt.rgba(0, 0, 0, 0.25)
              border.width: 1
              border.color: themePicker._mode === modelData
                ? (themePicker.colors ? themePicker.colors.primary : "#7986cb")
                : Qt.rgba(1, 1, 1, 0.12)

              Text {
                anchors.centerIn: parent
                text: modelData.toUpperCase()
                font.family: Style.fontFamily; font.pixelSize: 8; font.weight: Font.Bold; font.letterSpacing: 0.6
                color: themePicker._mode === modelData
                  ? (themePicker.colors ? themePicker.colors.primaryText : "#fff")
                  : Qt.rgba(1, 1, 1, 0.6)
              }

              MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                onClicked: { themePicker._mode = modelData; themePicker._palettes = ({}); themePicker._refresh() }
              }
            }
          }
        }
      }

      Row {
        anchors.left: parent.left
        spacing: 0

        Item { width: 96; height: 14 }

        Repeater {
          model: themePicker._activeIndices
          Item {
            required property int index
            width: 96; height: 14
            Text {
              anchors.centerIn: parent
              text: "i" + index
              font.family: Style.fontFamilyCode; font.pixelSize: 9; font.weight: Font.Bold
              color: themePicker.colors ? Qt.rgba(themePicker.colors.surfaceText.r, themePicker.colors.surfaceText.g, themePicker.colors.surfaceText.b, 0.45) : Qt.rgba(1,1,1,0.4)
            }
          }
        }
      }

      Repeater {
        model: themePicker._schemes

        Row {
          required property string modelData
          readonly property string _scheme: modelData
          width: parent.width
          spacing: 0

          Item {
            id: schemeLabel
            width: 96; height: 28
            Text {
              anchors.left: parent.left
              anchors.verticalCenter: parent.verticalCenter
              text: schemeLabel.parent._scheme.replace("scheme-", "").replace("-", " ")
              font.family: Style.fontFamily; font.pixelSize: 10; font.weight: Font.Medium
              color: themePicker.colors ? themePicker.colors.surfaceText : "#fff"
              elide: Text.ElideRight
              width: parent.width
            }
          }

          Repeater {
            model: themePicker._activeIndices

            Rectangle {
              id: cell
              required property int index
              readonly property int _idx: index
              readonly property string _scheme: cell.parent._scheme
              readonly property var _entry: themePicker._palettes[themePicker._key(_scheme, _idx)] || null
              readonly property bool _ready: _entry && _entry.primary && _entry.primary.length > 0
              readonly property bool _missing: _entry && _entry._missing === true
              readonly property bool _isCurrent: _scheme === Config.matugenScheme
                                                && _idx === Config.matugenColorIndex
                                                && themePicker._mode === Config.matugenMode

              width: 96; height: 32; radius: 4
              color: _isCurrent
                ? (themePicker.colors ? Qt.rgba(themePicker.colors.primary.r, themePicker.colors.primary.g, themePicker.colors.primary.b, 0.18) : Qt.rgba(0.4, 0.5, 0.7, 0.18))
                : "transparent"
              border.width: _isCurrent ? 1 : 0
              border.color: _isCurrent ? (themePicker.colors ? themePicker.colors.primary : "#7986cb") : "transparent"
              opacity: _missing ? 0.25 : 1.0

              Row {
                anchors.centerIn: parent
                spacing: -8
                visible: cell._ready

                Repeater {
                  model: cell._ready
                    ? [cell._entry.primary, cell._entry.secondary, cell._entry.tertiary]
                    : ["", "", ""]

                  Item {
                    required property string modelData
                    required property int index
                    width: 26; height: 22
                    z: cell._isCurrent ? (10 - index) : (5 - index)

                    Canvas {
                      anchors.fill: parent
                      property color cFill: parent.modelData && parent.modelData.length > 0 ? parent.modelData : Qt.rgba(1, 1, 1, 0.08)
                      property color bgCol: themePicker.colors ? themePicker.colors.surfaceContainer : Qt.rgba(0.1, 0.12, 0.18, 1.0)
                      onCFillChanged: requestPaint()
                      onBgColChanged: requestPaint()
                      onPaint: {
                        var ctx = getContext("2d")
                        ctx.clearRect(0, 0, width, height)
                        var sk = 8
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
                        ctx.fillStyle = cFill
                        ctx.beginPath()
                        ctx.moveTo(iSk + inset, inset)
                        ctx.lineTo(width - inset, inset)
                        ctx.lineTo(width - inset - iSk, height - inset)
                        ctx.lineTo(inset, height - inset)
                        ctx.closePath()
                        ctx.fill()
                      }
                    }
                  }
                }
              }

              Text {
                anchors.centerIn: parent
                text: cell._missing ? "—" : "…"
                font.family: Style.fontFamilyCode; font.pixelSize: 10
                color: Qt.rgba(1, 1, 1, 0.4)
                visible: !cell._ready
              }

              MouseArea {
                anchors.fill: parent
                enabled: cell._ready
                cursorShape: cell._ready ? Qt.PointingHandCursor : Qt.ArrowCursor
                onClicked: {
                  themePicker.applied(cell._scheme, themePicker._mode, cell._idx)
                  themePicker.close()
                }
              }
            }
          }
        }
      }

      Item {
        width: parent.width
        height: 30

        FilterButton {
          anchors.right: parent.right
          anchors.verticalCenter: parent.verticalCenter
          colors: themePicker.colors
          label: "CLOSE"
          skew: 8; height: 24
          onClicked: { themePicker.cancelled(); themePicker.close() }
        }
      }
    }
  }

  Keys.onEscapePressed: function(event) {
    event.accepted = true
    themePicker.cancelled()
    themePicker.close()
  }
}
