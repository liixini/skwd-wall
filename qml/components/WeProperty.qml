import QtQuick
import QtQuick.Shapes
import ".."
import "../wallpaper"

Rectangle {
  id: root
  property var colors
  property var property
  property var value

  readonly property string _type: property && property.type ? (property.type + "").toLowerCase() : ""
  readonly property string _rawText: property && property.text ? (property.text + "") : ""
  readonly property string _label: _stripHtml(_rawText) || (property ? (property.name || "") : "")
  readonly property bool _isLabelOnly: _type === "" || _type === "text" || _type === "group"
  readonly property bool _isUnsupported: _type === "usershortcut" || _type === "scenetexture" || _type === "file" || _type === "directory"

  function _stripHtml(s) {
    if (!s) return ""
    return ("" + s)
      .replace(/<[^>]+>/g, "")
      .replace(/&nbsp;/g, " ")
      .replace(/&amp;/g, "&")
      .replace(/&lt;/g, "<")
      .replace(/&gt;/g, ">")
      .replace(/\s+/g, " ")
      .trim()
  }

  width: parent ? parent.width : 240
  height: visible ? (_isLabelOnly ? labelOnly.height + 8 : controlled.height + 8) : 0
  color: _isLabelOnly
         ? "transparent"
         : (colors ? Qt.rgba(colors.surfaceVariant.r, colors.surfaceVariant.g, colors.surfaceVariant.b, 0.35)
                   : Qt.rgba(1, 1, 1, 0.05))
  radius: 4
  visible: !_isUnsupported && !(property && property.visible === false)

  Item {
    id: labelOnly
    visible: root._isLabelOnly
    anchors.left: parent.left
    anchors.right: parent.right
    anchors.top: parent.top
    anchors.leftMargin: 4
    anchors.rightMargin: 4
    height: labelOnlyText.implicitHeight + 6

    Text {
      id: labelOnlyText
      anchors.left: parent.left
      anchors.right: parent.right
      anchors.verticalCenter: parent.verticalCenter
      text: root._label
      font.family: Style.fontFamilyCode
      font.pixelSize: 10
      font.bold: root._type === "group"
      color: root.colors ? root.colors.surfaceText : "#cccccc"
      opacity: root._type === "group" ? 0.9 : 0.6
      wrapMode: Text.WordWrap
      maximumLineCount: 3
      elide: Text.ElideRight
    }
  }

  Column {
    id: controlled
    visible: !root._isLabelOnly && !root._isUnsupported
    anchors.left: parent.left
    anchors.right: parent.right
    anchors.top: parent.top
    anchors.leftMargin: 8
    anchors.rightMargin: 8
    anchors.topMargin: 4
    spacing: 3

    Row {
      width: parent.width
      spacing: 6

      Text {
        anchors.verticalCenter: parent.verticalCenter
        text: root._label
        font.family: Style.fontFamilyCode
        font.pixelSize: 10
        color: root.colors ? root.colors.surfaceText : "#cccccc"
        opacity: 0.8
        width: parent.width - controlLoader.width - parent.spacing
        elide: Text.ElideRight
      }

      Loader {
        id: controlLoader
        anchors.verticalCenter: parent.verticalCenter
        sourceComponent: {
          if (root._type === "bool") return boolControl
          if (root._type === "combo") return comboControl
          return null
        }
      }
    }

    Loader {
      width: parent.width
      sourceComponent: {
        if (root._type === "slider") return sliderControl
        if (root._type === "color") return colorControl
        if (root._type === "textinput") return textControl
        return null
      }
    }
  }

  Component {
    id: boolControl
    Rectangle {
      width: 28; height: 16; radius: 8
      color: root.value ? (root.colors ? root.colors.primary : "#7986cb")
                        : (root.colors ? Qt.rgba(root.colors.outline.r, root.colors.outline.g, root.colors.outline.b, 0.4)
                                       : Qt.rgba(1, 1, 1, 0.15))
      Rectangle {
        width: 12; height: 12; radius: 6
        anchors.verticalCenter: parent.verticalCenter
        x: root.value ? parent.width - width - 2 : 2
        color: "white"
        Behavior on x { NumberAnimation { duration: 120 } }
      }
      MouseArea {
        anchors.fill: parent
        cursorShape: Qt.PointingHandCursor
        onClicked: root.value = !root.value
      }
    }
  }

  Component {
    id: sliderControl
    Item {
      width: parent ? parent.width : 200
      height: 18
      readonly property real _min: (root.property && typeof root.property.min === "number") ? root.property.min : 0
      readonly property real _max: (root.property && typeof root.property.max === "number") ? root.property.max : 1
      readonly property real _range: Math.max(0.0001, _max - _min)
      readonly property real _v: (typeof root.value === "number") ? root.value : _min
      Rectangle {
        anchors.verticalCenter: parent.verticalCenter
        width: parent.width - readout.width - 6
        height: 4; radius: 2
        color: root.colors ? Qt.rgba(root.colors.outline.r, root.colors.outline.g, root.colors.outline.b, 0.35)
                           : Qt.rgba(1, 1, 1, 0.15)
        Rectangle {
          width: Math.max(0, Math.min(parent.width, parent.width * (parent.parent._v - parent.parent._min) / parent.parent._range))
          height: parent.height; radius: parent.radius
          color: root.colors ? root.colors.primary : "#7986cb"
        }
        Rectangle {
          width: 10; height: 10; radius: 5
          anchors.verticalCenter: parent.verticalCenter
          x: Math.max(0, Math.min(parent.width - width, (parent.width - width) * (parent.parent._v - parent.parent._min) / parent.parent._range))
          color: "white"
        }
        MouseArea {
          anchors.fill: parent
          cursorShape: Qt.PointingHandCursor
          onPressed: function(mouse) { update(mouse.x) }
          onPositionChanged: function(mouse) { if (pressed) update(mouse.x) }
          function update(px) {
            var t = Math.max(0, Math.min(1, px / width))
            root.value = parent.parent._min + t * parent.parent._range
          }
        }
      }
      Text {
        id: readout
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        text: (typeof root.value === "number") ? root.value.toFixed(2) : ""
        font.family: Style.fontFamilyCode; font.pixelSize: 9
        color: root.colors ? root.colors.surfaceText : "#cccccc"
        opacity: 0.7
        width: 36
        horizontalAlignment: Text.AlignRight
      }
    }
  }

  Component {
    id: comboControl
    Flow {
      width: parent ? parent.width : 200
      spacing: 3
      Repeater {
        model: (root.property && root.property.options) ? root.property.options : []
        delegate: FilterButton {
          colors: root.colors
          label: modelData.label || modelData.value
          skew: 6
          height: 20
          isActive: root.value === modelData.value || root.value === modelData.label
          onClicked: root.value = (modelData.value !== undefined) ? modelData.value : modelData.label
        }
      }
    }
  }

  Component {
    id: colorControl
    Row {
      spacing: 6
      width: parent ? parent.width : 200
      function _parseColor(s) {
        if (!s) return Qt.rgba(0.5, 0.5, 0.5, 1)
        var parts = (s + "").trim().split(/\s+/).map(parseFloat)
        if (parts.length >= 3 && parts.every(function(n) { return !isNaN(n) }))
          return Qt.rgba(Math.max(0, Math.min(1, parts[0])),
                          Math.max(0, Math.min(1, parts[1])),
                          Math.max(0, Math.min(1, parts[2])),
                          1)
        return Qt.rgba(0.5, 0.5, 0.5, 1)
      }
      Rectangle {
        width: 14; height: 14; radius: 3
        anchors.verticalCenter: parent.verticalCenter
        color: parent._parseColor(root.value)
        border.color: Qt.rgba(0, 0, 0, 0.3); border.width: 1
      }
      Rectangle {
        width: parent.width - 20
        height: 16; radius: 3
        anchors.verticalCenter: parent.verticalCenter
        color: root.colors ? Qt.rgba(root.colors.surface.r, root.colors.surface.g, root.colors.surface.b, 0.6)
                           : Qt.rgba(0, 0, 0, 0.3)
        TextInput {
          anchors.fill: parent
          anchors.leftMargin: 4; anchors.rightMargin: 4
          verticalAlignment: TextInput.AlignVCenter
          text: root.value || ""
          font.family: Style.fontFamilyCode; font.pixelSize: 9
          color: root.colors ? root.colors.surfaceText : "#cccccc"
          selectByMouse: true
          onEditingFinished: root.value = text
        }
      }
    }
  }

  Component {
    id: textControl
    Rectangle {
      width: parent ? parent.width : 200
      height: 18; radius: 3
      color: root.colors ? Qt.rgba(root.colors.surface.r, root.colors.surface.g, root.colors.surface.b, 0.6)
                         : Qt.rgba(0, 0, 0, 0.3)
      TextInput {
        anchors.fill: parent
        anchors.leftMargin: 4; anchors.rightMargin: 4
        verticalAlignment: TextInput.AlignVCenter
        text: root.value || ""
        font.family: Style.fontFamilyCode; font.pixelSize: 10
        color: root.colors ? root.colors.surfaceText : "#cccccc"
        selectByMouse: true
        onEditingFinished: root.value = text
      }
    }
  }
}
