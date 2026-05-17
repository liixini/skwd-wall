import QtQuick
import ".."

Item {
    id: root

    property var colors
    property color value: "#ffffff"

    signal commit(color v)

    property real _hue: 0
    property real _sat: 0
    property real _val: 1
    property bool _internal: false

    function _syncFromValue() {
        if (_internal) return
        var c = Qt.color(value)
        var r = c.r, g = c.g, b = c.b
        var max = Math.max(r, g, b), min = Math.min(r, g, b)
        var v = max
        var d = max - min
        var s = max === 0 ? 0 : d / max
        var h = 0
        if (d !== 0) {
            if (max === r)      h = ((g - b) / d) % 6
            else if (max === g) h = (b - r) / d + 2
            else                h = (r - g) / d + 4
            h *= 60
            if (h < 0) h += 360
        }
        _hue = h; _sat = s; _val = v
    }

    function _push() {
        _internal = true
        var c = Qt.hsva(_hue / 360, _sat, _val, 1)
        value = c
        root.commit(c)
        _internal = false
    }

    onValueChanged: _syncFromValue()
    Component.onCompleted: _syncFromValue()

    width: 220; height: 240

    Item {
        id: ring
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: 200; height: 200

        readonly property real cx: width / 2
        readonly property real cy: height / 2
        readonly property real outerR: Math.min(width, height) / 2 - 2
        readonly property real innerR: outerR - 18
        readonly property real svSide: innerR * Math.SQRT2 - 16

        Canvas {
            id: hueCanvas
            anchors.fill: parent
            onPaint: {
                var ctx = getContext("2d")
                ctx.clearRect(0, 0, width, height)
                var steps = 360
                var lineW = ring.outerR - ring.innerR
                var midR = (ring.outerR + ring.innerR) / 2
                ctx.lineWidth = lineW
                for (var a = 0; a < steps; a++) {
                    var rad0 = (a - 0.5) * Math.PI / 180
                    var rad1 = (a + 0.5) * Math.PI / 180
                    ctx.strokeStyle = Qt.hsla(a / 360, 1, 0.5, 1)
                    ctx.beginPath()
                    ctx.arc(ring.cx, ring.cy, midR, rad0, rad1)
                    ctx.stroke()
                }
            }
        }

        MouseArea {
            id: ringMA
            anchors.fill: parent
            preventStealing: true
            onPressed: function(m) { _update(m.x, m.y) }
            onPositionChanged: function(m) { if (pressed) _update(m.x, m.y) }
            function _update(x, y) {
                var dx = x - ring.cx, dy = y - ring.cy
                var dist = Math.sqrt(dx*dx + dy*dy)
                if (dist < ring.innerR - 4 || dist > ring.outerR + 4) return
                var h = Math.atan2(dy, dx) * 180 / Math.PI
                if (h < 0) h += 360
                root._hue = h
                root._push()
                svCanvas.requestPaint()
            }
        }

        Rectangle {
            width: 10; height: 10; radius: 5
            x: ring.cx + Math.cos(root._hue * Math.PI / 180) * ((ring.outerR + ring.innerR) / 2) - 5
            y: ring.cy + Math.sin(root._hue * Math.PI / 180) * ((ring.outerR + ring.innerR) / 2) - 5
            color: "transparent"
            border.color: "#ffffff"; border.width: 2
        }

        Rectangle {
            id: svRect
            width: ring.svSide; height: ring.svSide
            anchors.centerIn: parent
            color: "#000000"
            border.width: 0

            Canvas {
                id: svCanvas
                anchors.fill: parent
                onPaint: {
                    var ctx = getContext("2d")
                    var w = width, h = height
                    var img = ctx.createImageData(w, h)
                    var hueRgb = Qt.hsla(root._hue / 360, 1, 0.5, 1)
                    for (var y = 0; y < h; y++) {
                        var v = 1 - y / h
                        for (var x = 0; x < w; x++) {
                            var s = x / w
                            var r = (1 - s) * v + s * hueRgb.r * v
                            var g = (1 - s) * v + s * hueRgb.g * v
                            var b = (1 - s) * v + s * hueRgb.b * v
                            var i = (y * w + x) * 4
                            img.data[i]     = Math.round(r * 255)
                            img.data[i + 1] = Math.round(g * 255)
                            img.data[i + 2] = Math.round(b * 255)
                            img.data[i + 3] = 255
                        }
                    }
                    ctx.putImageData(img, 0, 0)
                }
            }

            MouseArea {
                anchors.fill: parent
                preventStealing: true
                onPressed: function(m) { _set(m.x, m.y) }
                onPositionChanged: function(m) { if (pressed) _set(m.x, m.y) }
                function _set(x, y) {
                    var xn = Math.max(0, Math.min(1, x / width))
                    var yn = Math.max(0, Math.min(1, y / height))
                    root._sat = xn
                    root._val = 1 - yn
                    root._push()
                }
            }

            Rectangle {
                width: 10; height: 10; radius: 5
                x: root._sat * parent.width - 5
                y: (1 - root._val) * parent.height - 5
                color: "transparent"
                border.color: "#ffffff"; border.width: 2
            }
        }
    }

    Row {
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: 8

        Rectangle {
            width: 22; height: 22; radius: 4
            color: root.value
            border.color: root.colors ? Qt.rgba(root.colors.outline.r, root.colors.outline.g, root.colors.outline.b, 0.4) : "#888"
            border.width: 1
        }

        Rectangle {
            width: 90; height: 22; radius: 4
            color: root.colors ? Qt.rgba(root.colors.surfaceContainer.r, root.colors.surfaceContainer.g, root.colors.surfaceContainer.b, 0.6) : "#333"
            border.width: hexIn.activeFocus ? 1 : 0
            border.color: root.colors ? root.colors.primary : "#888"
            TextInput {
                id: hexIn
                anchors.fill: parent
                anchors.leftMargin: 6
                anchors.rightMargin: 6
                verticalAlignment: TextInput.AlignVCenter
                horizontalAlignment: TextInput.AlignHCenter
                font.family: Style.fontFamilyCode
                font.pixelSize: 11
                color: root.colors ? root.colors.surfaceText : "#fff"
                clip: true
                selectByMouse: true
                text: {
                    var c = Qt.color(root.value)
                    return "#" + _hex(c.r) + _hex(c.g) + _hex(c.b)
                }
                function _commit() {
                    var t = text.trim().replace(/^#/, "")
                    if (!/^[0-9a-fA-F]{6}$/.test(t)) return
                    var c = Qt.color("#" + t)
                    root.value = c
                    root.commit(c)
                }
                Keys.onReturnPressed: { _commit(); focus = false }
                onEditingFinished: _commit()
            }
        }
    }

    function _hex(f) {
        var n = Math.round(f * 255)
        var s = n.toString(16)
        return s.length === 1 ? "0" + s : s
    }
}
