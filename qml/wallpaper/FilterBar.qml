import QtQuick
import ".."
import "../services"

Item {
    id: filterBar

    property var colors
    property var service
    property bool settingsOpen: false
    property bool ollamaActive: false
    property bool wallhavenBrowserOpen: false
    property bool steamWorkshopBrowserOpen: false
    property bool cacheLoading: false
    property int cacheProgress: 0
    property int cacheTotal: 0
    property bool matugenRunning: false
    property int matugenProgress: 0
    property int matugenTotal: 0
    property int ollamaProgress: 0
    property int ollamaTotal: 0
    property string ollamaEta: ""
    property string ollamaLogLine: ""
    property bool videoConvertRunning: false
    property int videoConvertProgress: 0
    property int videoConvertTotal: 0
    property string videoConvertFile: ""
    property bool imageOptimizeRunning: false
    property int imageOptimizeProgress: 0
    property int imageOptimizeTotal: 0
    property string imageOptimizeFile: ""

    property bool tagCloudOpen: false

    signal settingsToggled()
    signal wallhavenToggled()
    signal steamWorkshopToggled()
    signal tagCloudToggled()

    readonly property int _skew: 10

    width: filterRow.width
    height: filterRow.height

    Row {
        id: filterRow
        anchors.centerIn: parent
        spacing: -_skew

        Repeater {
            model: [
                { type: "", label: "ALL" },
                { type: "static", label: "PIC" },
                { type: "video", label: "VID" },
                { type: "we", label: "WE" }
            ]

            FilterButton {
                colors: filterBar.colors
                label: modelData.label
                isActive: filterBar.service ? filterBar.service.selectedTypeFilter === modelData.type : false
                onClicked: {
                    if (isActive) filterBar.service.selectedTypeFilter = ""
                    else filterBar.service.selectedTypeFilter = modelData.type
                }
            }
        }

        Repeater {
            model: [
                { mode: "date", icon: "󰃰", label: "Newest" },
                { mode: "color", icon: "󰏘", label: "Color" }
            ]

            FilterButton {
                colors: filterBar.colors
                icon: modelData.icon
                tooltip: modelData.label
                isActive: filterBar.service ? filterBar.service.sortMode === modelData.mode : false
                onClicked: {
                    filterBar.service.sortMode = modelData.mode
                    filterBar.service.updateFilteredModel()
                }
            }
        }

        FilterButton {
            colors: filterBar.colors
            icon: "󰋑"
            tooltip: "Favourites"
            isActive: filterBar.service ? filterBar.service.favouriteFilterActive : false
            onClicked: filterBar.service.favouriteFilterActive = !filterBar.service.favouriteFilterActive
        }

        Repeater {
            model: 13

            Item {
                width: 28; height: 24
                readonly property int filterValue: index < 12 ? index : 99
                readonly property bool isSelected: filterBar.service ? filterBar.service.selectedColorFilter === filterValue : false
                readonly property color hueColor: index === 12 ? Qt.hsla(0, 0, 0.45, 1.0) : Qt.hsla(index / 12.0, 0.65, 0.45, 1.0)
                readonly property color hueBright: index === 12 ? Qt.hsla(0, 0, 0.6, 1.0) : Qt.hsla(index / 12.0, 0.75, 0.55, 1.0)
                readonly property bool isHovered: _colorMouse.containsMouse
                z: isSelected ? 10 : (isHovered ? 5 : 1)

                Canvas {
                    id: _colorCanvas
                    anchors.fill: parent
                    scale: parent.isSelected ? 1.15 : 1.0
                    Behavior on scale { NumberAnimation { duration: Style.animVeryFast; easing.type: Easing.OutBack } }
                    property color cFill: parent.isSelected ? parent.hueBright : parent.hueColor
                    property color bgCol: filterBar.colors ? filterBar.colors.surfaceContainer : Qt.rgba(0.1, 0.12, 0.18, 1.0)
                    property bool sel: parent.isSelected
                    property bool hov: parent.isHovered
                    onCFillChanged: requestPaint()
                    onSelChanged: requestPaint()
                    onHovChanged: requestPaint()
                    onBgColChanged: requestPaint()
                    onPaint: {
                        var ctx = getContext("2d")
                        ctx.clearRect(0, 0, width, height)
                        var sk = filterBar._skew
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
                        ctx.fillStyle = hov ? Qt.lighter(cFill, 1.2) : cFill
                        ctx.beginPath()
                        ctx.moveTo(iSk + inset, inset)
                        ctx.lineTo(width - inset, inset)
                        ctx.lineTo(width - inset - iSk, height - inset)
                        ctx.lineTo(inset, height - inset)
                        ctx.closePath()
                        ctx.fill()
                    }
                }

                MouseArea {
                    id: _colorMouse
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    hoverEnabled: true
                    onClicked: {
                        if (parent.isSelected) filterBar.service.selectedColorFilter = -1
                        else filterBar.service.selectedColorFilter = parent.filterValue
                    }
                }
            }
        }

        FilterButton {
            colors: filterBar.colors
            icon: "\u{f0349}"
            tooltip: "Tags"
            isActive: filterBar.tagCloudOpen
            onClicked: filterBar.tagCloudToggled()
        }

        FilterButton {
            visible: Config.wallhavenEnabled
            colors: filterBar.colors
            icon: "\u{f01da}"
            tooltip: "Browse wallhaven.cc"
            isActive: filterBar.wallhavenBrowserOpen
            onClicked: filterBar.wallhavenToggled()
        }

        FilterButton {
            visible: Config.steamEnabled
            colors: filterBar.colors
            icon: "󰓓"
            tooltip: "Browse Steam Workshop"
            isActive: filterBar.steamWorkshopBrowserOpen
            onClicked: filterBar.steamWorkshopToggled()
        }

        FilterButton {
            colors: filterBar.colors
            icon: "\u{f0493}"
            tooltip: "Settings"
            isActive: filterBar.settingsOpen
            onClicked: filterBar.settingsToggled()
        }

        Item {
            width: _countLabel.implicitWidth + 24 + filterBar._skew
            height: 24

            Canvas {
                anchors.fill: parent
                property color fillColor: filterBar.colors ? Qt.rgba(filterBar.colors.surfaceContainer.r, filterBar.colors.surfaceContainer.g, filterBar.colors.surfaceContainer.b, 0.85) : Qt.rgba(0.1, 0.12, 0.18, 0.85)
                property color strokeColor: filterBar.colors ? Qt.rgba(filterBar.colors.primary.r, filterBar.colors.primary.g, filterBar.colors.primary.b, 0.15) : Qt.rgba(1, 1, 1, 0.08)
                onFillColorChanged: requestPaint()
                onStrokeColorChanged: requestPaint()
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    var sk = filterBar._skew
                    ctx.fillStyle = fillColor
                    ctx.strokeStyle = strokeColor
                    ctx.lineWidth = 1
                    ctx.beginPath()
                    ctx.moveTo(sk, 0)
                    ctx.lineTo(width, 0)
                    ctx.lineTo(width - sk, height)
                    ctx.lineTo(0, height)
                    ctx.closePath()
                    ctx.fill()
                    ctx.stroke()
                }
            }

            Text {
                id: _countLabel
                anchors.centerIn: parent
                text: {
                    if (!filterBar.service) return "0"
                    var fc = filterBar.service.filteredModel.count
                    var tc = filterBar.service._wallpaperData.length
                    return fc + (fc !== tc ? "/" + tc : "")
                }
                font.family: Style.fontFamily
                font.pixelSize: 10
                font.weight: Font.Bold
                font.letterSpacing: 0.5
                color: filterBar.colors ? Qt.rgba(filterBar.colors.surfaceText.r, filterBar.colors.surfaceText.g, filterBar.colors.surfaceText.b, 0.5) : Qt.rgba(1, 1, 1, 0.4)
            }
        }

        Item {
            visible: filterBar.cacheLoading || filterBar.ollamaActive || filterBar.matugenRunning || filterBar.videoConvertRunning || filterBar.imageOptimizeRunning
            width: visible ? (_statusRow.width + 24 + filterBar._skew) : 0
            height: 24

            Canvas {
                anchors.fill: parent
                visible: parent.visible
                property color fillColor: filterBar.colors ? Qt.rgba(filterBar.colors.surfaceContainer.r, filterBar.colors.surfaceContainer.g, filterBar.colors.surfaceContainer.b, 0.85) : Qt.rgba(0.1, 0.12, 0.18, 0.85)
                property color strokeColor: filterBar.colors ? Qt.rgba(filterBar.colors.primary.r, filterBar.colors.primary.g, filterBar.colors.primary.b, 0.15) : Qt.rgba(1, 1, 1, 0.08)
                onFillColorChanged: requestPaint()
                onStrokeColorChanged: requestPaint()
                onWidthChanged: requestPaint()
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    var sk = filterBar._skew
                    ctx.fillStyle = fillColor
                    ctx.strokeStyle = strokeColor
                    ctx.lineWidth = 1
                    ctx.beginPath()
                    ctx.moveTo(sk, 0)
                    ctx.lineTo(width, 0)
                    ctx.lineTo(width - sk, height)
                    ctx.lineTo(0, height)
                    ctx.closePath()
                    ctx.fill()
                    ctx.stroke()
                }
            }

            Row {
                id: _statusRow
                anchors.centerIn: parent
                spacing: 4

                Text {
                    text: "󰔟"
                    font.pixelSize: 11
                    font.family: Style.fontFamilyNerdIcons
                    color: filterBar.colors ? filterBar.colors.primary : Style.fallbackAccent
                    anchors.verticalCenter: parent.verticalCenter
                    RotationAnimation on rotation {
                        from: 0; to: 360; duration: 1200
                        loops: Animation.Infinite
                        running: filterBar.cacheLoading || filterBar.ollamaActive || filterBar.matugenRunning || filterBar.videoConvertRunning || filterBar.imageOptimizeRunning
                    }
                }

                Text {
                    text: {
                        var parts = []
                        if (filterBar.cacheLoading) {
                            if (filterBar.cacheTotal > 0)
                                parts.push("CACHE " + filterBar.cacheProgress + "/" + filterBar.cacheTotal)
                            else
                                parts.push("PROCESSING")
                        }
                        if (filterBar.ollamaActive) {
                            if (filterBar.ollamaTotal > 0)
                                parts.push("OLLAMA " + filterBar.ollamaProgress + "/" + filterBar.ollamaTotal)
                            else
                                parts.push("OLLAMA")
                        }
                        if (filterBar.matugenRunning) {
                            if (filterBar.matugenTotal > 0)
                                parts.push("MATUGEN " + filterBar.matugenProgress + "/" + filterBar.matugenTotal)
                            else
                                parts.push("MATUGEN")
                        }
                        if (filterBar.videoConvertRunning) {
                            if (filterBar.videoConvertTotal > 0)
                                parts.push("CONVERT " + filterBar.videoConvertProgress + "/" + filterBar.videoConvertTotal)
                            else
                                parts.push("CONVERT")
                        }
                        if (filterBar.imageOptimizeRunning) {
                            if (filterBar.imageOptimizeTotal > 0)
                                parts.push("OPTIMIZE " + filterBar.imageOptimizeProgress + "/" + filterBar.imageOptimizeTotal)
                            else
                                parts.push("OPTIMIZE")
                        }
                        return parts.join(" · ")
                    }
                    font.family: Style.fontFamily
                    font.pixelSize: 9
                    font.weight: Font.Bold
                    font.letterSpacing: 0.5
                    color: filterBar.colors ? Qt.rgba(filterBar.colors.primary.r, filterBar.colors.primary.g, filterBar.colors.primary.b, 0.8) : Qt.rgba(0.5, 0.76, 0.97, 0.8)
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
        }

        Item {
            visible: (filterBar.videoConvertRunning && filterBar.videoConvertFile !== "") || (filterBar.imageOptimizeRunning && filterBar.imageOptimizeFile !== "")
            width: visible ? (180 + 24 + filterBar._skew) : 0
            height: 24
            Behavior on width { NumberAnimation { duration: Style.animFast } }

            Canvas {
                anchors.fill: parent
                visible: parent.visible
                property color fillColor: filterBar.colors ? Qt.rgba(filterBar.colors.surfaceContainer.r, filterBar.colors.surfaceContainer.g, filterBar.colors.surfaceContainer.b, 0.85) : Qt.rgba(0.1, 0.12, 0.18, 0.85)
                property color strokeColor: filterBar.colors ? Qt.rgba(filterBar.colors.primary.r, filterBar.colors.primary.g, filterBar.colors.primary.b, 0.15) : Qt.rgba(1, 1, 1, 0.08)
                onFillColorChanged: requestPaint()
                onStrokeColorChanged: requestPaint()
                onWidthChanged: requestPaint()
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    var sk = filterBar._skew
                    ctx.fillStyle = fillColor
                    ctx.strokeStyle = strokeColor
                    ctx.lineWidth = 1
                    ctx.beginPath()
                    ctx.moveTo(sk, 0)
                    ctx.lineTo(width, 0)
                    ctx.lineTo(width - sk, height)
                    ctx.lineTo(0, height)
                    ctx.closePath()
                    ctx.fill()
                    ctx.stroke()
                }
            }

            Text {
                id: _convertLogText
                anchors.centerIn: parent
                width: Math.min(implicitWidth, 180)
                text: filterBar.imageOptimizeRunning ? filterBar.imageOptimizeFile : filterBar.videoConvertFile
                font.family: Style.fontFamilyCode
                font.pixelSize: 8
                font.letterSpacing: 0.3
                elide: Text.ElideMiddle
                maximumLineCount: 1
                color: filterBar.colors ? Qt.rgba(filterBar.colors.surfaceText.r, filterBar.colors.surfaceText.g, filterBar.colors.surfaceText.b, 0.5) : Qt.rgba(1, 1, 1, 0.4)
            }
        }

        Item {
            visible: filterBar.ollamaActive && filterBar.ollamaLogLine !== ""
            width: visible ? (Math.min(_ollamaLogText.implicitWidth, 220) + 24 + filterBar._skew) : 0
            height: 24
            Behavior on width { NumberAnimation { duration: Style.animFast } }

            Canvas {
                anchors.fill: parent
                visible: parent.visible
                property color fillColor: filterBar.colors ? Qt.rgba(filterBar.colors.surfaceContainer.r, filterBar.colors.surfaceContainer.g, filterBar.colors.surfaceContainer.b, 0.85) : Qt.rgba(0.1, 0.12, 0.18, 0.85)
                property color strokeColor: filterBar.colors ? Qt.rgba(filterBar.colors.primary.r, filterBar.colors.primary.g, filterBar.colors.primary.b, 0.15) : Qt.rgba(1, 1, 1, 0.08)
                onFillColorChanged: requestPaint()
                onStrokeColorChanged: requestPaint()
                onWidthChanged: requestPaint()
                onPaint: {
                    var ctx = getContext("2d")
                    ctx.clearRect(0, 0, width, height)
                    var sk = filterBar._skew
                    ctx.fillStyle = fillColor
                    ctx.strokeStyle = strokeColor
                    ctx.lineWidth = 1
                    ctx.beginPath()
                    ctx.moveTo(sk, 0)
                    ctx.lineTo(width, 0)
                    ctx.lineTo(width - sk, height)
                    ctx.lineTo(0, height)
                    ctx.closePath()
                    ctx.fill()
                    ctx.stroke()
                }
            }

            Text {
                id: _ollamaLogText
                anchors.centerIn: parent
                width: Math.min(implicitWidth, 220)
                text: filterBar.ollamaLogLine
                font.family: Style.fontFamilyCode
                font.pixelSize: 8
                font.letterSpacing: 0.3
                elide: Text.ElideMiddle
                maximumLineCount: 1
                color: filterBar.colors ? Qt.rgba(filterBar.colors.surfaceText.r, filterBar.colors.surfaceText.g, filterBar.colors.surfaceText.b, 0.5) : Qt.rgba(1, 1, 1, 0.4)
            }
        }

        FilterButton {
            visible: Config.ollamaEnabled
            colors: filterBar.colors
            label: "O"
            tooltip: filterBar.ollamaActive ? "Stop Ollama scan" : "Start Ollama scan"
            isActive: filterBar.ollamaActive
            onClicked: {
                if (filterBar.ollamaActive) WallpaperAnalysisService.stop()
                else WallpaperAnalysisService.start()
            }
        }
    }
}
