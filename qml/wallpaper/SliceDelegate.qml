import QtQuick
import QtQuick.Shapes
import QtQuick.Effects
import QtQuick.Controls
import QtMultimedia
import ".."
import "../services"

Item {
    id: delegateItem

    property var colors
    property int expandedWidth: 768
    property int sliceWidth: 108
    property int skewOffset: 28
    property var service

    property int selectedIdx: -1
    property bool isCurrent: ListView.isCurrentItem
    property bool isHovered: itemMouseArea.containsMouse
    property bool flipped: false
    property var _backMeta: null
    readonly property var _listView: ListView.view

    onFlippedChanged: {
        if (flipped && model.type !== "we") {
            var key = ImageService.thumbKey(model.thumb, model.name)
            _backMeta = FileMetadataService.getMetadata(key)
            if (!_backMeta)
                FileMetadataService.probeIfNeeded(key, model.path, model.type === "video" ? "video" : "image")
        }
    }
    Connections {
        target: FileMetadataService
        enabled: delegateItem.flipped
        function onMetadataReady(key) {
            var myKey = ImageService.thumbKey(model.thumb, model.name)
            if (key === myKey)
                delegateItem._backMeta = FileMetadataService.getMetadata(key)
        }
    }

    readonly property real _skAbs: Math.abs(skewOffset)
    readonly property real _topLeft: skewOffset >= 0 ? _skAbs : 0
    readonly property real _topRight: skewOffset >= 0 ? width : width - _skAbs
    readonly property real _botRight: skewOffset >= 0 ? width - _skAbs : width
    readonly property real _botLeft: skewOffset >= 0 ? 0 : _skAbs

    property bool suppressWidthAnim: false
    property string videoPath: model.videoFile ? model.videoFile : ""
    property bool hasVideo: videoPath.length > 0 && Config.videoPreviewEnabled
    property bool videoActive: false

    width: isCurrent ? expandedWidth : sliceWidth
    height: _listView ? _listView.height : 0

    onIsCurrentChanged: {
        if (!isCurrent) flipped = false
        if (isCurrent && hasVideo) {
            videoDelayTimer.restart()
        } else {
            videoDelayTimer.stop()
            videoActive = false
        }
    }

    Timer {
        id: videoDelayTimer
        interval: 300
        onTriggered: delegateItem.videoActive = true
    }

    z: isCurrent ? 100 : (isHovered ? 90 : 50 - Math.min(Math.abs(index - (_listView ? _listView.currentIndex : 0)), 50))

    readonly property real _fadeZone: sliceWidth * 1.5
    readonly property real _center: _listView ? ((x - _listView.contentX) + width * 0.5) : (width * 0.5)
    opacity: _fadeZone > 0 ? Math.min(Math.min(1.0, Math.max(0.0, _center / _fadeZone)),
                                      Math.min(1.0, Math.max(0.0, ((_listView ? _listView.width : 0) - _center) / _fadeZone))) : 1.0
    Behavior on width {
        enabled: !suppressWidthAnim
        NumberAnimation { duration: Style.animNormal; easing.type: Easing.OutQuad }
    }

    containmentMask: Item {
        id: hitMask
        function contains(point) {
            var w = delegateItem.width
            var h = delegateItem.height
            if (h <= 0 || w <= 0) return false
            var t = point.y / h
            var leftX = delegateItem._topLeft * (1.0 - t) + delegateItem._botLeft * t
            var rightX = delegateItem._topRight * (1.0 - t) + delegateItem._botRight * t
            return point.x >= leftX && point.x <= rightX && point.y >= 0 && point.y <= h
        }
    }

    Loader {
        id: sharedVideoLoader
        width: delegateItem.width
        height: delegateItem.height
        active: delegateItem.videoActive
        visible: false
        layer.enabled: active

        sourceComponent: Video {
            anchors.fill: parent
            source: "file://" + delegateItem.videoPath
            fillMode: VideoOutput.PreserveAspectCrop
            loops: MediaPlayer.Infinite
            muted: true
            Component.onCompleted: play()
        }
    }

    Item {
        id: sharedMask
        width: delegateItem.width
        height: delegateItem.height
        visible: false
        layer.enabled: true
        layer.smooth: true
        Shape {
            anchors.fill: parent
            antialiasing: true
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                fillColor: "white"
                strokeColor: "transparent"
                startX: delegateItem._topLeft
                startY: 0
                PathLine { x: delegateItem._topRight; y: 0 }
                PathLine { x: delegateItem._botRight; y: delegateItem.height }
                PathLine { x: delegateItem._botLeft; y: delegateItem.height }
                PathLine { x: delegateItem._topLeft; y: 0 }
            }
        }
    }

    Item {
        id: flipContainer
        anchors.fill: parent
        transform: Rotation {
            id: flipRotation
            origin.x: flipContainer.width / 2
            origin.y: flipContainer.height / 2
            axis { x: 0; y: 1; z: 0 }
            angle: delegateItem.flipped ? 180 : 0
            Behavior on angle {
                NumberAnimation { duration: Style.animSlow; easing.type: Easing.InOutQuad }
            }
        }

    Item {
        id: frontFace
        anchors.fill: parent
        visible: flipRotation.angle < 90

    Shape {
        id: shadowShape
        z: -1
        x: delegateItem.isCurrent ? 4 : 2
        y: delegateItem.isCurrent ? 10 : 5
        width: delegateItem.width
        height: delegateItem.height
        opacity: delegateItem.isCurrent ? 0.5 : 0.3
        Behavior on x { NumberAnimation { duration: Style.animNormal } }
        Behavior on y { NumberAnimation { duration: Style.animNormal } }
        Behavior on opacity { NumberAnimation { duration: Style.animNormal } }
        ShapePath {
            fillColor: "#000000"
            strokeColor: "transparent"
            startX: delegateItem._topLeft
            startY: 0
            PathLine { x: delegateItem._topRight; y: 0 }
            PathLine { x: delegateItem._botRight; y: delegateItem.height }
            PathLine { x: delegateItem._botLeft; y: delegateItem.height }
            PathLine { x: delegateItem._topLeft; y: 0 }
        }
    }

    Item {
        id: imageContainer
        anchors.fill: parent
        Image {
            id: thumbImage
            anchors.fill: parent
            source: model.thumb ? ("file://" + model.thumb) : ""
            fillMode: Image.PreserveAspectCrop
            smooth: true
            asynchronous: true
            cache: false
            sourceSize.width: 400
            sourceSize.height: 720
        }

        Rectangle {
            anchors.fill: parent
            visible: thumbImage.status !== Image.Ready
            color: delegateItem.colors ? Qt.rgba(delegateItem.colors.surfaceVariant.r, delegateItem.colors.surfaceVariant.g, delegateItem.colors.surfaceVariant.b, 0.8) : Qt.rgba(0.18, 0.20, 0.25, 0.8)
        }

        Rectangle {
            anchors.fill: parent
            color: Qt.rgba(0, 0, 0, delegateItem.isCurrent ? 0 : (delegateItem.isHovered ? 0.15 : 0.4))
            Behavior on color { ColorAnimation { duration: Style.animNormal } }
        }
        layer.enabled: true
        layer.smooth: true
        layer.effect: MultiEffect {
            maskEnabled: true
            maskSource: sharedMask
            maskThresholdMin: 0.3
            maskSpreadAtMin: 0.3
        }
    }

    Item {
        id: videoOverlay
        anchors.fill: parent
        visible: sharedVideoLoader.active && sharedVideoLoader.status === Loader.Ready

        ShaderEffectSource {
            anchors.fill: parent
            sourceItem: sharedVideoLoader
            live: true
        }

        layer.enabled: true
        layer.smooth: true
        layer.effect: MultiEffect {
            maskEnabled: true
            maskSource: sharedMask
            maskThresholdMin: 0.3
            maskSpreadAtMin: 0.3
        }
    }

    Shape {
        id: glowBorder
        anchors.fill: parent
        antialiasing: true
        preferredRendererType: Shape.CurveRenderer
        opacity: 1.0
        ShapePath {
            fillColor: "transparent"
            strokeColor: delegateItem.isCurrent
                ? (delegateItem.colors ? delegateItem.colors.primary : "#8BC34A")
                : (delegateItem.isHovered
                    ? Qt.rgba(delegateItem.colors ? delegateItem.colors.primary.r : 0.5, delegateItem.colors ? delegateItem.colors.primary.g : 0.76, delegateItem.colors ? delegateItem.colors.primary.b : 0.29, 0.4)
                    : Qt.rgba(0, 0, 0, 0.6))
            Behavior on strokeColor { ColorAnimation { duration: Style.animNormal } }
            strokeWidth: delegateItem.isCurrent ? 3 : 1
            startX: delegateItem._topLeft
            startY: 0
            PathLine { x: delegateItem._topRight; y: 0 }
            PathLine { x: delegateItem._botRight; y: delegateItem.height }
            PathLine { x: delegateItem._botLeft; y: delegateItem.height }
            PathLine { x: delegateItem._topLeft; y: 0 }
        }
    }

    Rectangle {
        id: videoIndicator
        anchors.top: parent.top
        anchors.topMargin: 10
        x: delegateItem.skewOffset >= 0
            ? parent.width - width - 10
            : 10
        width: 22
        height: 22
        radius: 11
        color: delegateItem.videoActive ? (delegateItem.colors ? delegateItem.colors.primary : Style.fallbackAccent) : Qt.rgba(0, 0, 0, 0.7)
        border.width: 1
        border.color: delegateItem.videoActive
            ? "transparent"
            : (delegateItem.colors ? Qt.rgba(delegateItem.colors.primary.r, delegateItem.colors.primary.g, delegateItem.colors.primary.b, 0.6) : Qt.rgba(1, 1, 1, 0.4))
        visible: delegateItem.hasVideo
        z: 10

        Behavior on color { ColorAnimation { duration: Style.animNormal } }

        Text {
            anchors.centerIn: parent
            anchors.horizontalCenterOffset: 1
            text: "▶"
            font.pixelSize: 9
            color: delegateItem.videoActive
                ? (delegateItem.colors ? delegateItem.colors.primaryText : "#000")
                : (delegateItem.colors ? delegateItem.colors.primary : Style.fallbackAccent)
        }
    }

    Item {
        id: typeBadge
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 8
        property real skew: 4
        property bool onRight: delegateItem.skewOffset >= 0
        property real _tl: onRight ? skew : 0
        property real _tr: onRight ? width : width - skew
        property real _br: onRight ? width - skew : width
        property real _bl: onRight ? 0 : skew
        width: typeBadgeText.implicitWidth + 16 + skew
        height: 16
        z: 10
        x: onRight
            ? parent.width - width - delegateItem._skAbs - 8
            : delegateItem._skAbs + 8

        Shape {
            anchors.fill: parent
            ShapePath {
                fillColor: Qt.rgba(0, 0, 0, 0.75)
                strokeColor: delegateItem.colors ? Qt.rgba(delegateItem.colors.primary.r, delegateItem.colors.primary.g, delegateItem.colors.primary.b, 0.4) : Qt.rgba(1, 1, 1, 0.2)
                strokeWidth: 1
                startX: typeBadge._tl; startY: 0
                PathLine { x: typeBadge._tr; y: 0 }
                PathLine { x: typeBadge._br; y: typeBadge.height }
                PathLine { x: typeBadge._bl; y: typeBadge.height }
                PathLine { x: typeBadge._tl; y: 0 }
            }
        }

        Text {
            id: typeBadgeText
            anchors.centerIn: parent
            text: model.type === "static" ? "PIC" : ((model.type === "video" || model.videoFile) ? "VID" : "WE")
            font.family: Style.fontFamily
            font.pixelSize: 9
            font.weight: Font.Bold
            font.letterSpacing: 0.5
            color: delegateItem.colors ? delegateItem.colors.tertiary : "#8bceff"
        }
    }

    Row {
        id: colorDotsRow
        z: 10
        anchors.verticalCenter: typeBadge.verticalCenter
        opacity: typeBadge.opacity
        spacing: 4
        visible: Config.wallpaperColorDots && wallpaperColors !== undefined
        property var wallpaperColors: {
            if (!delegateItem.service) return undefined
            var key = model.weId ? model.weId : ImageService.thumbKey(model.thumb, model.name)
            if (!key) return undefined
            return delegateItem.service.matugenDb[key]
        }

        states: [
            State {
                name: "right"
                when: typeBadge.onRight
                AnchorChanges {
                    target: colorDotsRow
                    anchors.right: typeBadge.left
                    anchors.left: undefined
                }
                PropertyChanges {
                    target: colorDotsRow
                    anchors.rightMargin: 6
                }
            },
            State {
                name: "left"
                when: !typeBadge.onRight
                AnchorChanges {
                    target: colorDotsRow
                    anchors.left: typeBadge.right
                    anchors.right: undefined
                }
                PropertyChanges {
                    target: colorDotsRow
                    anchors.leftMargin: 6
                }
            }
        ]

        Repeater {
            model: ["primary", "tertiary", "secondary"]
            Rectangle {
                width: 10; height: 10; radius: 5
                color: colorDotsRow.wallpaperColors ? (colorDotsRow.wallpaperColors[modelData] ?? "#888") : "#888"
                border.width: 1; border.color: Qt.rgba(0, 0, 0, 0.5)
            }
        }
    }

    }

    Item {
        id: backFace
        anchors.fill: parent
        visible: flipRotation.angle >= 90
        transform: Rotation {
            origin.x: backFace.width / 2
            origin.y: backFace.height / 2
            axis { x: 0; y: 1; z: 0 }
            angle: 180
        }

        Item {
            id: backClip
            anchors.fill: parent

            Rectangle {
                anchors.fill: parent
                color: delegateItem.colors
                    ? delegateItem.colors.surfaceContainer
                    : "#1a1a2e"
            }

            ShaderEffectSource {
                anchors.fill: parent
                sourceItem: sharedVideoLoader
                live: true
                visible: delegateItem.videoActive && delegateItem.flipped && sharedVideoLoader.status === Loader.Ready
                opacity: 0.25
            }

            Image {
                anchors.fill: parent
                source: "file://" + model.thumb
                fillMode: Image.PreserveAspectCrop
                opacity: 0.12
                visible: !(delegateItem.videoActive && delegateItem.flipped)
                cache: false; asynchronous: true
                sourceSize.width: 120
                sourceSize.height: 216
            }

            Column {
                anchors.fill: parent
                anchors.leftMargin: delegateItem._skAbs + 14
                anchors.rightMargin: delegateItem._skAbs + 14
                anchors.topMargin: 16
                anchors.bottomMargin: 16
                spacing: 10

                Text {
                    width: parent.width
                    text: model.name.replace(/\.[^/.]+$/, "").toUpperCase()
                    color: delegateItem.colors ? delegateItem.colors.tertiary : "#8bceff"
                    font.family: Style.fontFamily
                    font.pixelSize: 13
                    font.weight: Font.Bold
                    font.letterSpacing: 1
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.Wrap
                    elide: Text.ElideRight
                    maximumLineCount: 2
                }

                Row {
                    width: parent.width
                    spacing: 0
                    visible: model.type !== "we"
                    layoutDirection: Qt.LeftToRight

                    Text {
                        text: FileMetadataService.formatExt(model.name)
                        color: delegateItem.colors ? Qt.rgba(delegateItem.colors.tertiary.r, delegateItem.colors.tertiary.g, delegateItem.colors.tertiary.b, 0.6) : Qt.rgba(1,1,1,0.35)
                        font.family: Style.fontFamily; font.pixelSize: 10; font.weight: Font.Medium; font.letterSpacing: 0.8
                    }
                    Text {
                        text: "  \u2022  "
                        color: Qt.rgba(1, 1, 1, 0.15)
                        font.family: Style.fontFamily; font.pixelSize: 10
                    }
                    Text {
                        text: delegateItem._backMeta ? (delegateItem._backMeta.width + " \u00d7 " + delegateItem._backMeta.height) : "\u2013"
                        color: delegateItem.colors ? Qt.rgba(delegateItem.colors.tertiary.r, delegateItem.colors.tertiary.g, delegateItem.colors.tertiary.b, 0.6) : Qt.rgba(1,1,1,0.35)
                        font.family: Style.fontFamily; font.pixelSize: 10; font.weight: Font.Medium; font.letterSpacing: 0.5
                    }
                    Text {
                        text: "  \u2022  "
                        color: Qt.rgba(1, 1, 1, 0.15)
                        font.family: Style.fontFamily; font.pixelSize: 10
                    }
                    Text {
                        text: delegateItem._backMeta ? FileMetadataService.formatSize(delegateItem._backMeta.filesize) : "\u2013"
                        color: delegateItem.colors ? Qt.rgba(delegateItem.colors.tertiary.r, delegateItem.colors.tertiary.g, delegateItem.colors.tertiary.b, 0.6) : Qt.rgba(1,1,1,0.35)
                        font.family: Style.fontFamily; font.pixelSize: 10; font.weight: Font.Medium; font.letterSpacing: 0.5
                    }
                }

                Item {
                    width: parent.width; height: 28

                    Text {
                        anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
                        text: "FAVOURITE"
                        color: delegateItem.colors ? delegateItem.colors.tertiary : "#8bceff"
                        font.family: Style.fontFamily; font.pixelSize: 11
                        font.weight: Font.Medium; font.letterSpacing: 0.5
                    }

                    Item {
                        id: favToggle
                        anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                        width: 48; height: 24
                        property bool checked: false
                        Component.onCompleted: {
                            var key = (model.weId || "") !== "" ? model.weId : model.name
                            checked = delegateItem.service ? !!delegateItem.service.favouritesDb[key] : false
                        }
                        Connections {
                            target: delegateItem
                            function onFlippedChanged() {
                                if (delegateItem.flipped) {
                                    var key = (model.weId || "") !== "" ? model.weId : model.name
                                    favToggle.checked = delegateItem.service ? !!delegateItem.service.favouritesDb[key] : false
                                }
                            }
                        }
                        Canvas {
                            anchors.fill: parent
                            property bool isOn: favToggle.checked
                            property color fillColor: isOn
                                ? (delegateItem.colors ? delegateItem.colors.primary : Style.fallbackAccent)
                                : Qt.rgba(1, 1, 1, 0.15)
                            onFillColorChanged: requestPaint()
                            onIsOnChanged: requestPaint()
                            onPaint: {
                                var ctx = getContext("2d"); ctx.clearRect(0, 0, width, height)
                                var sk = 8; ctx.fillStyle = fillColor; ctx.beginPath()
                                ctx.moveTo(sk, 0); ctx.lineTo(width, 0)
                                ctx.lineTo(width - sk, height); ctx.lineTo(0, height)
                                ctx.closePath(); ctx.fill()
                            }
                        }
                        Canvas {
                            width: 22; height: 18; y: 3
                            x: favToggle.checked ? parent.width - width - 4 : 4
                            Behavior on x { NumberAnimation { duration: Style.animFast; easing.type: Easing.OutCubic } }
                            property color knobColor: favToggle.checked
                                ? (delegateItem.colors ? delegateItem.colors.primaryText : "#000")
                                : (delegateItem.colors ? delegateItem.colors.surfaceText : "#fff")
                            onKnobColorChanged: requestPaint()
                            onPaint: {
                                var ctx = getContext("2d"); ctx.clearRect(0, 0, width, height)
                                var sk = 5; ctx.fillStyle = knobColor; ctx.beginPath()
                                ctx.moveTo(sk, 0); ctx.lineTo(width, 0)
                                ctx.lineTo(width - sk, height); ctx.lineTo(0, height)
                                ctx.closePath(); ctx.fill()
                            }
                        }
                        MouseArea {
                            anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                            onClicked: { favToggle.checked = !favToggle.checked; delegateItem.service.toggleFavourite(model.name, model.weId || "") }
                        }
                    }
                }

                Rectangle { width: parent.width; height: 1; color: Qt.rgba(1, 1, 1, 0.08) }

                Item {
                    id: backAddTagRow
                    width: parent.width; height: 22

                    Rectangle {
                        anchors.fill: parent
                        color: addTagField.activeFocus
                            ? (delegateItem.colors ? Qt.rgba(delegateItem.colors.surface.r, delegateItem.colors.surface.g, delegateItem.colors.surface.b, 0.5) : Qt.rgba(0, 0, 0, 0.3))
                            : "transparent"
                        border.width: 1
                        border.color: addTagField.activeFocus
                            ? (delegateItem.colors ? Qt.rgba(delegateItem.colors.primary.r, delegateItem.colors.primary.g, delegateItem.colors.primary.b, 0.5) : Qt.rgba(1, 1, 1, 0.3))
                            : (delegateItem.colors ? Qt.rgba(delegateItem.colors.outline.r, delegateItem.colors.outline.g, delegateItem.colors.outline.b, 0.2) : Qt.rgba(1, 1, 1, 0.1))
                        Behavior on color { ColorAnimation { duration: Style.animVeryFast } }
                        Behavior on border.color { ColorAnimation { duration: Style.animVeryFast } }
                    }

                    TextInput {
                        id: addTagField
                        anchors.fill: parent; anchors.leftMargin: 8; anchors.rightMargin: 8
                        verticalAlignment: TextInput.AlignVCenter
                        font.family: Style.fontFamily; font.pixelSize: 10; font.letterSpacing: 0.3
                        color: delegateItem.colors ? delegateItem.colors.surfaceText : "#fff"
                        clip: true
                        Keys.onReturnPressed: {
                            var tag = text.trim().toLowerCase()
                            if (tag.length > 0) {
                                var tags = delegateItem.service.getWallpaperTags(backTagsSection.wpName, backTagsSection.wpWeId).slice()
                                if (tags.indexOf(tag) === -1) { tags.push(tag); delegateItem.service.setWallpaperTags(backTagsSection.wpName, backTagsSection.wpWeId, tags) }
                                text = ""
                            }
                        }
                        Keys.onEscapePressed: { text = ""; if (delegateItem._listView) delegateItem._listView.forceActiveFocus() }

                        Text {
                            anchors.fill: parent; verticalAlignment: Text.AlignVCenter
                            text: "+ ADD TAG"
                            font.family: Style.fontFamily; font.pixelSize: 10; font.letterSpacing: 1
                            color: delegateItem.colors ? Qt.rgba(delegateItem.colors.surfaceText.r, delegateItem.colors.surfaceText.g, delegateItem.colors.surfaceText.b, 0.25) : Qt.rgba(1, 1, 1, 0.2)
                            visible: !parent.text && !parent.activeFocus
                        }
                    }

                    MouseArea {
                        anchors.fill: parent; cursorShape: Qt.IBeamCursor; z: -1
                        onClicked: addTagField.forceActiveFocus()
                    }
                }

                Item {
                    id: backTagsSection
                    width: parent.width
                    height: parent.height - y - backActionRow.height - parent.spacing
                    clip: true

                    property string wpName: model.name
                    property string wpWeId: model.weId || ""
                    property string wpThumb: model.thumb || ""
                    property var currentTags: {
                        if (!delegateItem.flipped) return []
                        var db = delegateItem.service ? delegateItem.service.tagsDb : null
                        if (!db) return []
                        var key = backTagsSection.wpWeId
                            ? backTagsSection.wpWeId
                            : ImageService.thumbKey(backTagsSection.wpThumb, backTagsSection.wpName)
                        return db[key] || []
                    }

                    Flickable {
                        anchors.fill: parent
                        contentHeight: backTagsFlowInner.implicitHeight
                        clip: true
                        flickableDirection: Flickable.VerticalFlick
                        boundsBehavior: Flickable.StopAtBounds

                        Flow {
                            id: backTagsFlowInner
                            width: parent.width
                            spacing: 6

                            Repeater {
                                model: backTagsSection.currentTags

                                Rectangle {
                                    property bool hovered: tagRemoveArea.containsMouse
                                    width: tagLabelText.implicitWidth + 28
                                    height: 26
                                    radius: 4
                                    color: hovered
                                        ? (delegateItem.colors ? Qt.rgba(delegateItem.colors.surfaceVariant.r, delegateItem.colors.surfaceVariant.g, delegateItem.colors.surfaceVariant.b, 0.5) : Qt.rgba(1, 1, 1, 0.15))
                                        : "transparent"
                                    border.width: 1
                                    border.color: hovered
                                        ? (delegateItem.colors ? Qt.rgba(delegateItem.colors.primary.r, delegateItem.colors.primary.g, delegateItem.colors.primary.b, 0.7) : Qt.rgba(1, 1, 1, 0.3))
                                        : (delegateItem.colors ? Qt.rgba(delegateItem.colors.outline.r, delegateItem.colors.outline.g, delegateItem.colors.outline.b, 0.5) : Qt.rgba(1, 1, 1, 0.15))
                                    Behavior on color { ColorAnimation { duration: Style.animVeryFast } }
                                    Behavior on border.color { ColorAnimation { duration: Style.animVeryFast } }

                                    transform: Matrix4x4 {
                                        matrix: Qt.matrix4x4(
                                            1, -0.08, 0, 0,
                                            0, 1,     0, 0,
                                            0, 0,     1, 0,
                                            0, 0,     0, 1)
                                    }

                                    Text {
                                        id: tagLabelText
                                        anchors.left: parent.left; anchors.leftMargin: 8
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: modelData.toUpperCase()
                                        color: delegateItem.colors ? delegateItem.colors.tertiary : "#8bceff"
                                        font.family: Style.fontFamily; font.pixelSize: 11
                                        font.weight: Font.Medium; font.letterSpacing: 0.5
                                    }

                                    Text {
                                        anchors.right: parent.right; anchors.rightMargin: 6
                                        anchors.verticalCenter: parent.verticalCenter
                                        text: "\u{f0156}"
                                        font.family: Style.fontFamilyNerdIcons; font.pixelSize: 10
                                        color: parent.hovered ? (delegateItem.colors ? delegateItem.colors.primary : "#ff6b6b") : Qt.rgba(1, 1, 1, 0.25)
                                        Behavior on color { ColorAnimation { duration: Style.animVeryFast } }
                                    }

                                    MouseArea {
                                        id: tagRemoveArea
                                        anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            var tags = delegateItem.service.getWallpaperTags(backTagsSection.wpName, backTagsSection.wpWeId).slice()
                                            var idx = tags.indexOf(modelData)
                                            if (idx !== -1) tags.splice(idx, 1)
                                            delegateItem.service.setWallpaperTags(backTagsSection.wpName, backTagsSection.wpWeId, tags)
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Text {
                        anchors.centerIn: parent
                        visible: backTagsSection.currentTags.length === 0
                        text: "NO TAGS"
                        color: Qt.rgba(1, 1, 1, 0.15)
                        font.family: Style.fontFamily; font.pixelSize: 11; font.letterSpacing: 2
                    }
                }

                Row {
                    id: backActionRow
                    width: parent.width; height: 30
                    spacing: 6

                    ActionButton {
                        width: model.type === "we" ? (parent.width - parent.spacing * 2) / 3 : (parent.width - parent.spacing) / 2
                        colors: delegateItem.colors
                        icon: "\u{f0208}"; label: "VIEW"
                        skew: Math.abs(delegateItem.skewOffset) * 0.4
                        onClicked: {
                            var dir = model.path.substring(0, model.path.lastIndexOf("/"))
                            Qt.openUrlExternally("file://" + dir)
                            delegateItem.flipped = false
                        }
                    }

                    ActionButton {
                        width: model.type === "we" ? (parent.width - parent.spacing * 2) / 3 : (parent.width - parent.spacing) / 2
                        colors: delegateItem.colors
                        icon: "\u{f0a79}"; label: "DELETE"; danger: true
                        skew: Math.abs(delegateItem.skewOffset) * 0.4
                        onClicked: {
                            var idx = index
                            delegateItem.service.deleteWallpaperItem(model.type, model.name, model.weId || "")
                            var newIdx = Math.min(idx, delegateItem.service.filteredModel.count - 1)
                            if (delegateItem._listView) {
                                delegateItem._listView.currentIndex = -1
                                delegateItem._listView.currentIndex = newIdx
                                delegateItem._listView.positionViewAtIndex(newIdx, ListView.Center)
                            }
                        }
                    }

                    ActionButton {
                        visible: model.type === "we"
                        width: visible ? (parent.width - parent.spacing * 2) / 3 : 0
                        colors: delegateItem.colors
                        icon: "\u{f0bef}"; label: "STEAM"
                        skew: Math.abs(delegateItem.skewOffset) * 0.4
                        onClicked: { delegateItem.service.openSteamPage(model.weId || ""); delegateItem.flipped = false }
                    }
                }
            }

            MouseArea {
                anchors.fill: parent
                z: -1
                onClicked: delegateItem.flipped = false
            }

            layer.enabled: true
            layer.smooth: true
            layer.effect: MultiEffect {
                maskEnabled: true
                maskSource: sharedMask
                maskThresholdMin: 0.3
                maskSpreadAtMin: 0.3
            }
        }

        Shape {
            anchors.fill: parent
            antialiasing: true
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                fillColor: "transparent"
                strokeColor: delegateItem.colors ? delegateItem.colors.primary : "#8BC34A"
                strokeWidth: 2
                startX: delegateItem._topLeft
                startY: 0
                PathLine { x: delegateItem._topRight; y: 0 }
                PathLine { x: delegateItem._botRight; y: delegateItem.height }
                PathLine { x: delegateItem._botLeft; y: delegateItem.height }
                PathLine { x: delegateItem._topLeft; y: 0 }
            }
        }
    }

    }

    MouseArea {
        id: itemMouseArea
        anchors.fill: parent
        hoverEnabled: !delegateItem.flipped
        acceptedButtons: delegateItem.flipped ? Qt.RightButton : (Qt.LeftButton | Qt.RightButton)
        cursorShape: delegateItem.flipped ? Qt.ArrowCursor : Qt.PointingHandCursor
        onPositionChanged: function(mouse) {
            if (delegateItem.flipped) return
            if (!delegateItem._listView) return
            var globalPos = mapToItem(delegateItem._listView, mouse.x, mouse.y)
            var dx = Math.abs(globalPos.x - delegateItem._listView.lastMouseX)
            var dy = Math.abs(globalPos.y - delegateItem._listView.lastMouseY)
            if (dx > 2 || dy > 2) {
                delegateItem._listView.lastMouseX = globalPos.x
                delegateItem._listView.lastMouseY = globalPos.y
                delegateItem._listView.keyboardNavActive = false
                delegateItem._listView.currentIndex = index
            }
        }
        onClicked: function(mouse) {
            if (mouse.button === Qt.RightButton) {
                if (delegateItem._listView) delegateItem._listView.currentIndex = index
                delegateItem.flipped = !delegateItem.flipped
            } else if (!delegateItem.flipped) {
                if (delegateItem.isCurrent) {
                    console.log("SliceDelegate: applying", model.type, model.path || model.weId)
                    if (model.type === "we") {
                        delegateItem.service.applyWE(model.weId)
                    } else if (model.type === "video") {
                        delegateItem.service.applyVideo(model.path)
                    } else {
                        delegateItem.service.applyStatic(model.path)
                    }
                } else {
                    if (delegateItem._listView) delegateItem._listView.currentIndex = index
                }
            }
        }
    }
}
