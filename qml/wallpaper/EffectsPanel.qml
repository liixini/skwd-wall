import QtQuick
import ".."
import "../components"
import "../services"

Item {
    id: panel

    property var colors
    property bool effectsOpen: false
    property string selectedPath: ""

    property var _effects: []
    property string _effectId: ""
    property var _paramValues: ({})
    property string _appliedPath: ""
    property string _appliedName: ""
    property string _previewPath: ""
    property string _statusText: ""

    readonly property string _sourcePath: selectedPath.length > 0 ? selectedPath : _appliedPath
    readonly property string _sourceName: {
        var p = _sourcePath
        return p.length > 0 ? p.substring(p.lastIndexOf("/") + 1) : ""
    }
    readonly property bool _sourceFromSelection: selectedPath.length > 0
    readonly property string _displayPath: _previewPath.length > 0 ? _previewPath : _sourcePath

    readonly property var _selectedEffect: {
        for (var i = 0; i < _effects.length; i++) {
            if (_effects[i].id === _effectId) return _effects[i]
        }
        return null
    }

    signal closeRequested()

    function _s(v) { return v * Config.uiScale }

    function _hex(c) {
        if (typeof c === "string") return c
        function pair(f) {
            var n = Math.round(f * 255)
            var s = n.toString(16)
            return s.length === 1 ? "0" + s : s
        }
        return "#" + pair(c.r) + pair(c.g) + pair(c.b)
    }

    function _resetParams() {
        var v = {}
        var eff = _selectedEffect
        if (eff && eff.params) {
            for (var i = 0; i < eff.params.length; i++) {
                v[eff.params[i].id] = eff.params[i]["default"]
            }
        }
        _paramValues = v
    }

    function _setParam(id, value) {
        var v = {}
        for (var k in _paramValues) v[k] = _paramValues[k]
        v[id] = value
        _paramValues = v
        _schedulePreview()
    }

    function _effectModel() {
        var out = []
        for (var i = 0; i < _effects.length; i++) {
            out.push({ mode: _effects[i].id, label: _effects[i].label })
        }
        return out
    }

    function _outboundParams() {
        var out = {}
        for (var k in _paramValues) {
            var v = _paramValues[k]
            if (typeof v === "object" && v !== null && "r" in v && "g" in v && "b" in v) {
                out[k] = _hex(v)
            } else {
                out[k] = v
            }
        }
        return out
    }

    function _schedulePreview() {
        if (!_sourcePath || !_selectedEffect) return
        previewDebounce.restart()
    }

    function _launchPreview() {
        if (!_sourcePath || !_selectedEffect) return
        if (_previewPath.length > 0) {
            EffectsService.discard(_previewPath)
            _previewPath = ""
        }
        _statusText = ""
        EffectsService.preview(_effectId, _sourcePath, _outboundParams())
    }

    function _discardCurrent() {
        if (_previewPath.length > 0) {
            EffectsService.discard(_previewPath)
            _previewPath = ""
        }
    }

    function _cancel() {
        previewDebounce.stop()
        _discardCurrent()
        closeRequested()
    }

    function _save() {
        if (_previewPath.length === 0 || EffectsService.busy) return
        EffectsService.commit(_previewPath, _sourcePath, _effectId, _outboundParams())
    }

    function _saveAndApply() {
        if (_previewPath.length === 0 || EffectsService.busy) return
        panel._pendingApply = true
        EffectsService.commit(_previewPath, _sourcePath, _effectId, _outboundParams())
    }

    property bool _pendingApply: false
    property bool _didInitialPreview: false

    function _maybeInitialPreview() {
        if (_didInitialPreview) return
        if (_effects.length === 0 || !_sourcePath || !_selectedEffect) return
        _didInitialPreview = true
        _schedulePreview()
    }

    width: 540 * Config.uiScale
    implicitHeight: contentColumn.implicitHeight

    Timer {
        id: previewDebounce
        interval: 350
        repeat: false
        onTriggered: panel._launchPreview()
    }

    Connections {
        target: DaemonClient
        function onWallpaperApplied(type, name, path, weId, key) {
            panel._appliedPath = path || ""
            panel._appliedName = name || (path ? path.substring(path.lastIndexOf("/") + 1) : "")
        }
    }

    Connections {
        target: EffectsService
        function onPreviewed(p) {
            panel._previewPath = p
            panel._statusText = ""
        }
        function onCommitted(p) {
            panel._previewPath = ""
            if (panel._pendingApply) {
                panel._pendingApply = false
                DaemonClient.applyStatic(p, [], [])
            }
            panel._statusText = "Saved to " + p
        }
        function onDiscarded() { }
        function onFailed(error) {
            panel._pendingApply = false
            panel._statusText = "Failed: " + error
        }
    }

    Component.onCompleted: {
        DaemonClient.call("effects.list", {}, function(result, err) {
            if (!err && result && result.effects) {
                panel._effects = result.effects
                if (panel._effects.length > 0) {
                    panel._effectId = panel._effects[0].id
                    panel._resetParams()
                }
            }
            panel._maybeInitialPreview()
        })
        DaemonClient.call("status", {}, function(result, err) {
            if (!err && result) {
                var p = result.current_wallpaper || ""
                if (p) {
                    panel._appliedPath = p
                    panel._appliedName = p.substring(p.lastIndexOf("/") + 1)
                }
            }
            panel._maybeInitialPreview()
        })
    }

    Column {
        id: contentColumn
        anchors.fill: parent
        spacing: _s(8)

        Rectangle {
            width: parent.width
            height: _s(240)
            radius: _s(8)
            color: panel.colors ? Qt.rgba(panel.colors.surfaceContainer.r, panel.colors.surfaceContainer.g, panel.colors.surfaceContainer.b, 0.6) : Qt.rgba(0.1, 0.12, 0.18, 0.6)
            border.width: 1
            border.color: panel._previewPath.length > 0
                ? (panel.colors ? Qt.rgba(panel.colors.primary.r, panel.colors.primary.g, panel.colors.primary.b, 0.5) : Qt.rgba(0.5, 0.7, 1.0, 0.5))
                : (panel.colors ? Qt.rgba(panel.colors.outline.r, panel.colors.outline.g, panel.colors.outline.b, 0.2) : Qt.rgba(1, 1, 1, 0.08))
            Behavior on border.color { ColorAnimation { duration: 200 } }
            clip: true

            Item {
                id: previewBox
                anchors.fill: parent
                anchors.margins: 1

                property string targetPath: panel._displayPath

                Image {
                    id: layerLower
                    anchors.fill: parent
                    fillMode: Image.PreserveAspectFit
                    asynchronous: true
                    smooth: true
                    cache: true
                    sourceSize.width: 1280
                }

                Image {
                    id: layerUpper
                    anchors.fill: parent
                    fillMode: Image.PreserveAspectFit
                    asynchronous: true
                    smooth: true
                    cache: true
                    sourceSize.width: 1280
                    opacity: 0
                    source: previewBox.targetPath ? "file://" + previewBox.targetPath : ""

                    onStatusChanged: {
                        if (status === Image.Ready && previewBox.targetPath.length > 0) {
                            if (layerLower.source == "" || layerLower.source.toString() === "") {
                                layerLower.source = source
                                opacity = 0
                            } else {
                                crossfade.restart()
                            }
                        }
                    }
                }

                NumberAnimation {
                    id: crossfade
                    target: layerUpper
                    property: "opacity"
                    from: 0
                    to: 1
                    duration: 280
                    easing.type: Easing.OutCubic
                    onStopped: {
                        layerLower.source = layerUpper.source
                        layerUpper.opacity = 0
                    }
                }
            }

            Rectangle {
                visible: panel._previewPath.length > 0
                anchors.top: parent.top
                anchors.right: parent.right
                anchors.margins: _s(6)
                width: previewLabel.implicitWidth + _s(10)
                height: previewLabel.implicitHeight + _s(4)
                radius: _s(3)
                color: panel.colors ? Qt.rgba(panel.colors.primary.r, panel.colors.primary.g, panel.colors.primary.b, 0.85) : Qt.rgba(0.5, 0.7, 1.0, 0.85)
                Text {
                    id: previewLabel
                    anchors.centerIn: parent
                    text: "PREVIEW"
                    font.family: Style.fontFamily
                    font.pixelSize: _s(9)
                    font.weight: Font.Bold
                    font.letterSpacing: 1.0
                    color: panel.colors ? panel.colors.onPrimary : "white"
                }
            }

            Rectangle {
                visible: EffectsService.busy
                anchors.bottom: parent.bottom
                anchors.right: parent.right
                anchors.margins: _s(6)
                width: busyLabel.implicitWidth + _s(10)
                height: busyLabel.implicitHeight + _s(4)
                radius: _s(3)
                color: Qt.rgba(0, 0, 0, 0.6)
                Text {
                    id: busyLabel
                    anchors.centerIn: parent
                    text: "RENDERING"
                    font.family: Style.fontFamily
                    font.pixelSize: _s(9)
                    font.weight: Font.Bold
                    font.letterSpacing: 1.0
                    color: "white"
                }
            }

            Text {
                visible: !panel._sourcePath
                anchors.centerIn: parent
                text: "Hover or focus a wallpaper, or apply one as the active wallpaper."
                font.family: Style.fontFamily
                font.pixelSize: _s(11)
                color: panel.colors ? Qt.rgba(panel.colors.surfaceText.r, panel.colors.surfaceText.g, panel.colors.surfaceText.b, 0.45) : Qt.rgba(1, 1, 1, 0.35)
            }
        }

        SettingsCard {
            colors: panel.colors
            title: "Effects"
            subtitle: panel._sourcePath.length > 0
                ? ((panel._sourceFromSelection ? "Selected: " : "Active: ") + panel._sourceName)
                : "Powered by gowall."

            titleAction: Rectangle {
                width: _s(24); height: _s(24); radius: _s(12)
                color: closeMA.containsMouse
                    ? (panel.colors ? Qt.rgba(panel.colors.error.r, panel.colors.error.g, panel.colors.error.b, 0.2) : Qt.rgba(1, 0.3, 0.3, 0.2))
                    : "transparent"
                Text {
                    anchors.centerIn: parent
                    text: "×"
                    font.family: Style.fontFamily
                    font.pixelSize: _s(16)
                    color: panel.colors ? panel.colors.surfaceText : "white"
                }
                MouseArea {
                    id: closeMA
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: panel._cancel()
                }
            }

            RowDropdown {
                colors: panel.colors
                title: "Effect"
                description: panel._selectedEffect ? panel._selectedEffect.description : ""
                value: panel._effectId
                model: panel._effectModel()
                onSelect: function(v) {
                    panel._effectId = v
                    panel._resetParams()
                    panel._schedulePreview()
                }
            }

            Repeater {
                model: panel._selectedEffect ? panel._selectedEffect.params : []

                delegate: Loader {
                    id: paramLoader
                    required property var modelData
                    property var pData: modelData
                    width: parent ? parent.width : 0
                    sourceComponent: {
                        switch (modelData.type) {
                            case "integer":  return _intComp
                            case "number":   return _numComp
                            case "dropdown": return _ddComp
                            case "color":    return _colorComp
                        }
                        return null
                    }
                }
            }

            RowAction {
                colors: panel.colors
                title: "Save"
                description: panel._statusText.length > 0
                    ? panel._statusText
                    : "Move the preview into your wallpapers/effects directory."
                enabled: panel._previewPath.length > 0 && !EffectsService.busy
                opacity: enabled ? 1.0 : 0.4
                onClicked: panel._save()
            }

            RowAction {
                colors: panel.colors
                title: "Save and apply"
                description: "Save the preview and set it as the active wallpaper."
                enabled: panel._previewPath.length > 0 && !EffectsService.busy
                opacity: enabled ? 1.0 : 0.4
                onClicked: panel._saveAndApply()
            }

            RowAction {
                colors: panel.colors
                title: "Cancel"
                description: "Discard the preview and close."
                onClicked: panel._cancel()
            }
        }
    }

    Component {
        id: _intComp
        RowInput {
            readonly property var pd: parent ? parent.pData : null
            colors: panel.colors
            title: pd ? pd.label : ""
            value: pd && panel._paramValues[pd.id] !== undefined ? panel._paramValues[pd.id] : (pd ? (pd["default"] || 0) : 0)
            min: pd && pd.min !== undefined ? pd.min : 0
            max: pd && pd.max !== undefined ? pd.max : 9999
            onCommit: function(v) { if (pd) panel._setParam(pd.id, v) }
        }
    }

    Component {
        id: _numComp
        RowInput {
            readonly property var pd: parent ? parent.pData : null
            colors: panel.colors
            title: pd ? pd.label : ""
            value: pd && panel._paramValues[pd.id] !== undefined ? panel._paramValues[pd.id] : (pd ? (pd["default"] || 0) : 0)
            min: pd && pd.min !== undefined ? pd.min : 0
            max: pd && pd.max !== undefined ? pd.max : 9999
            decimals: pd && pd.decimals !== undefined ? pd.decimals : 2
            onCommit: function(v) { if (pd) panel._setParam(pd.id, v) }
        }
    }

    Component {
        id: _ddComp
        RowDropdown {
            readonly property var pd: parent ? parent.pData : null
            colors: panel.colors
            title: pd ? pd.label : ""
            value: pd && panel._paramValues[pd.id] !== undefined ? panel._paramValues[pd.id] : (pd ? (pd["default"] || "") : "")
            model: pd && pd.options ? pd.options : []
            onSelect: function(v) { if (pd) panel._setParam(pd.id, v) }
        }
    }

    Component {
        id: _colorComp
        RowColor {
            readonly property var pd: parent ? parent.pData : null
            colors: panel.colors
            title: pd ? pd.label : ""
            value: {
                if (!pd) return "#000000"
                var v = panel._paramValues[pd.id]
                if (v === undefined) return pd["default"] || "#000000"
                return v
            }
            onCommit: function(c) { if (pd) panel._setParam(pd.id, c) }
        }
    }
}
