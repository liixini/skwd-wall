import QtQuick
import QtQuick.Shapes
import QtQuick.Effects
import ".."
import "../services"

Item {
    id: cell

    property var cellData
    property var colors
    property var itemData
    property bool hovered: false
    property real cloudOpacity: 1.0
    property int cellKey: 0
    property real _imageAlpha: 0.0
    property string _revealedSource: ""
    readonly property int _decodeW: ImageService.thumbWidth
    readonly property int _decodeH: ImageService.thumbHeight

    signal hoverChanged(bool isHover)
    signal activated(var item)

    function _startRevealForCurrentSource() {
        if (!thumb.source || thumb.source.length === 0) return
        if (cloudOpacity < 0.995) return
        if (_revealedSource === thumb.source) return
        _revealTimer.interval = 20 + ((Math.abs(cellKey) % 7) * 14)
        _revealTimer.restart()
    }

    onCloudOpacityChanged: {
        if (thumb.status === Image.Ready) _startRevealForCurrentSource()
    }

    onHoveredChanged: {
        if (cell.hovered) _preheatTimer.restart()
        else _preheatTimer.stop()
    }

    Timer {
        id: _preheatTimer
        interval: 120
        repeat: false
        onTriggered: {
            if (cell.itemData && cell.itemData.path)
                DaemonClient.preheat(cell.itemData.path)
        }
    }

    x: cellData ? cellData.bbox.x : 0
    y: cellData ? cellData.bbox.y : 0
    width: cellData ? cellData.bbox.w : 0
    height: cellData ? cellData.bbox.h : 0
    visible: itemData !== null
    opacity: cloudOpacity * _imageAlpha

    Behavior on _imageAlpha {
        NumberAnimation { duration: Style.animNormal; easing.type: Easing.OutCubic }
    }

    
    Item {
        id: maskShape
        anchors.fill: parent
        visible: false
        layer.enabled: true

        Shape {
            anchors.fill: parent
            antialiasing: true
            ShapePath {
                fillColor: "white"
                strokeColor: "transparent"
                strokeWidth: 0
                startX: cell.cellData && cell.cellData.polygon.length > 0
                        ? (cell.cellData.polygon[0][0] - cell.cellData.bbox.x) : 0
                startY: cell.cellData && cell.cellData.polygon.length > 0
                        ? (cell.cellData.polygon[0][1] - cell.cellData.bbox.y) : 0
                PathPolyline {
                    path: {
                        if (!cell.cellData) return []
                        var poly = cell.cellData.polygon
                        var bx = cell.cellData.bbox.x
                        var by = cell.cellData.bbox.y
                        var pts = []
                        for (var i = 1; i < poly.length; i++) {
                            pts.push(Qt.point(poly[i][0] - bx, poly[i][1] - by))
                        }
                        pts.push(Qt.point(poly[0][0] - bx, poly[0][1] - by))
                        return pts
                    }
                }
            }
        }
    }

    
    Rectangle {
        anchors.fill: parent
        color: cell.colors ? Qt.rgba(cell.colors.surfaceContainer.r, cell.colors.surfaceContainer.g, cell.colors.surfaceContainer.b, 0.5)
                           : Qt.rgba(0.08, 0.09, 0.12, 0.5)
        layer.enabled: true
        layer.effect: MultiEffect {
            maskEnabled: true
            maskSource: maskShape
        }
    }

    Image {
        id: thumb
        anchors.fill: parent
        fillMode: Image.PreserveAspectCrop
        cache: true
        asynchronous: true
        smooth: true
        source: cell.itemData && cell.itemData.thumb ? ImageService.fileUrl(cell.itemData.thumb) : ""
        sourceSize.width: cell._decodeW
        sourceSize.height: cell._decodeH
        visible: false

        onSourceChanged: {
            cell._imageAlpha = 0
            cell._revealedSource = ""
            if (status === Image.Ready) cell._startRevealForCurrentSource()
        }
        onStatusChanged: {
            if (status === Image.Ready) cell._startRevealForCurrentSource()
        }
    }

    Timer {
        id: _revealTimer
        interval: 30
        repeat: false
        onTriggered: {
            cell._revealedSource = thumb.source
            cell._imageAlpha = 1
        }
    }

    MultiEffect {
        anchors.fill: parent
        source: thumb
        maskEnabled: true
        maskSource: maskShape
        brightness: cell.hovered ? 0.05 : -0.05
        saturation: cell.hovered ? 0.0 : -0.1
    }

    Rectangle {
        anchors.fill: parent
        visible: cell.hovered && cell.colors !== null
        color: cell.colors ? Qt.rgba(cell.colors.primary.r, cell.colors.primary.g, cell.colors.primary.b, 0.18)
                           : "transparent"
        layer.enabled: cell.hovered
        layer.effect: MultiEffect {
            maskEnabled: true
            maskSource: maskShape
        }
    }

    Shape {
        anchors.fill: parent
        antialiasing: true
        visible: cell.hovered
        ShapePath {
            fillColor: "transparent"
            strokeColor: cell.colors ? cell.colors.primary : Qt.rgba(1, 1, 1, 0.6)
            strokeWidth: 2
            joinStyle: ShapePath.RoundJoin
            startX: cell.cellData && cell.cellData.polygon.length > 0
                    ? (cell.cellData.polygon[0][0] - cell.cellData.bbox.x) : 0
            startY: cell.cellData && cell.cellData.polygon.length > 0
                    ? (cell.cellData.polygon[0][1] - cell.cellData.bbox.y) : 0
            PathPolyline {
                path: {
                    if (!cell.cellData) return []
                    var poly = cell.cellData.polygon
                    var bx = cell.cellData.bbox.x
                    var by = cell.cellData.bbox.y
                    var pts = []
                    for (var i = 1; i < poly.length; i++) {
                        pts.push(Qt.point(poly[i][0] - bx, poly[i][1] - by))
                    }
                    pts.push(Qt.point(poly[0][0] - bx, poly[0][1] - by))
                    return pts
                }
            }
        }
    }

    function _pointInPoly(px, py) {
        if (!cellData) return false
        var poly = cellData.polygon
        var inside = false
        for (var i = 0, j = poly.length - 1; i < poly.length; j = i++) {
            var xi = poly[i][0], yi = poly[i][1]
            var xj = poly[j][0], yj = poly[j][1]
            var intersect = ((yi > py) !== (yj > py)) &&
                            (px < (xj - xi) * (py - yi) / ((yj - yi) || 1e-9) + xi)
            if (intersect) inside = !inside
        }
        return inside
    }

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        acceptedButtons: Qt.LeftButton
        propagateComposedEvents: true

        onPositionChanged: function(ev) {
            var gx = cell.x + ev.x
            var gy = cell.y + ev.y
            var inside = cell._pointInPoly(gx, gy)
            if (inside !== cell.hovered) cell.hoverChanged(inside)
        }
        onExited: {
            if (cell.hovered) cell.hoverChanged(false)
        }
        onClicked: function(ev) {
            var gx = cell.x + ev.x
            var gy = cell.y + ev.y
            if (cell._pointInPoly(gx, gy) && cell.itemData) cell.activated(cell.itemData)
        }
    }
}
