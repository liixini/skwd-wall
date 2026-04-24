import QtQuick
import QtQuick.Controls
import ".."

Rectangle {
    id: tagCloud

    property var colors
    property var service
    property bool tagCloudVisible: false

    readonly property bool searchFocused: tagSearchInput.activeFocus

    signal escapePressed()
    signal closeRequested()

    function reset() {
        tagSearchInput.text = ""
        _tagSearchQuery = ""
        if (service) {
            service.selectedTags = []
            service.updateFilteredModel(true)
        }
        _recomputeTags()
    }

    property real parentWidth: 800
    width: parentWidth
    Behavior on width { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }

    height: tagCloudVisible ? 154 * Config.uiScale : 0
    visible: tagCloudVisible
    radius: 16
    clip: true
    color: "transparent"

    Behavior on height { NumberAnimation { duration: Style.animNormal; easing.type: Easing.OutCubic } }

    property int _popularTagCount: service ? service.popularTags.length : 0
    property int _selectedTagCount: service ? service.selectedTags.length : 0
    on_PopularTagCountChanged: { console.log("[TagCloud] _popularTagCount changed to " + _popularTagCount); _recomputeTags() }
    on_SelectedTagCountChanged: { console.log("[TagCloud] _selectedTagCount changed to " + _selectedTagCount); _recomputeTags() }

    onTagCloudVisibleChanged: {
        console.log("[TagCloud] tagCloudVisible=" + tagCloudVisible + " service=" + (service ? "yes" : "null") + " popularTags=" + (service ? service.popularTags.length : "n/a"))
        if (tagCloudVisible) {
            tagCloudFlow._settled = false
            _entranceActive = true
            _recomputeTags()
            _entranceTimer.start()
            _focusTimer.start()
        } else {
            _entranceActive = false
        }
    }

    Timer { id: _focusTimer; interval: 0; onTriggered: tagSearchInput.forceActiveFocus() }

    onServiceChanged: {
        console.log("[TagCloud] serviceChanged: " + (service ? "got service, popularTags=" + service.popularTags.length + " tagsDb=" + Object.keys(service.tagsDb).length : "null"))
        if (service && tagCloudVisible)
            _recomputeTags()
    }

    MouseArea {
        anchors.fill: parent
        z: -1
        onClicked: function(mouse) { mouse.accepted = true }
        onPressed: function(mouse) { mouse.accepted = true }
    }

    property string _tagSearchQuery: ""
    property string _autoSuggestion: ""
    property var _visibleTagsCache: []
    property bool _syncingText: false
    property bool _tagsDirty: true
    property bool _entranceActive: false
    property var _pendingTagsCache: null

    Timer {
        id: _entranceTimer
        interval: 500
        onTriggered: tagCloud._entranceActive = false
    }

    Timer {
        id: _tagSearchDebounce
        interval: 250
        onTriggered: tagCloud._recomputeTags()
    }

    on_TagSearchQueryChanged: {
        _tagsDirty = true
        _tagSearchDebounce.restart()
    }

    function _stem(w) {
        if (w.length < 3) return w
        if (w.endsWith("ies") && w.length > 4) return w.slice(0, -3) + "y"
        if (w.endsWith("ves") && w.length > 4) return w.slice(0, -3) + "f"
        if (w.endsWith("ses") || w.endsWith("xes") || w.endsWith("zes") || w.endsWith("ches") || w.endsWith("shes"))
            return w.endsWith("ches") || w.endsWith("shes") ? w.slice(0, -2) : w.slice(0, -2)
        if (w.endsWith("ness") && w.length > 5) return w.slice(0, -4)
        if (w.endsWith("ment") && w.length > 5) return w.slice(0, -4)
        if (w.endsWith("ing") && w.length > 4) {
            var base = w.slice(0, -3)
            if (base.length > 1 && base[base.length - 1] === base[base.length - 2])
                return base.slice(0, -1)
            return base
        }
        if (w.endsWith("ed") && w.length > 3) {
            var b = w.slice(0, -2)
            if (b.length > 1 && b[b.length - 1] === b[b.length - 2])
                return b.slice(0, -1)
            return b
        }
        if (w.endsWith("er") && w.length > 3) return w.slice(0, -2)
        if (w.endsWith("ly") && w.length > 3) return w.slice(0, -2)
        if (w.endsWith("s") && !w.endsWith("ss") && w.length > 3) return w.slice(0, -1)
        return w
    }

    function _editDist(a, b) {
        if (a === b) return 0
        var m = a.length, n = b.length
        if (!m) return n
        if (!n) return m
        var prev = new Array(n + 1), curr = new Array(n + 1)
        for (var j = 0; j <= n; j++) prev[j] = j
        for (var i = 1; i <= m; i++) {
            curr[0] = i
            for (var j2 = 1; j2 <= n; j2++) {
                curr[j2] = a[i-1] === b[j2-1]
                    ? prev[j2-1]
                    : 1 + Math.min(prev[j2-1], prev[j2], curr[j2-1])
            }
            var tmp = prev; prev = curr; curr = tmp
        }
        return prev[n]
    }

    function _fuzzyMatch(tagName, query) {
        if (tagName.indexOf(query) !== -1) return true
        var st = _stem(tagName), sq = _stem(query)
        if (st === sq || st.indexOf(sq) !== -1 || sq.indexOf(st) !== -1) return true
        var maxDist = Math.min(sq.length, st.length) <= 4 ? 1 : 2
        return _editDist(st, sq) <= maxDist
    }

    function _recomputeTags() {
        var query = _tagSearchQuery
        var svc = service
        if (!svc) { console.log("[TagCloud] _recomputeTags: no service"); _visibleTagsCache = []; _tagsDirty = false; return }
        var selected = svc.selectedTags || []
        var popLen = (svc.popularTags || []).length
        var dbKeys = Object.keys(svc.tagsDb || {}).length
        console.log("[TagCloud] _recomputeTags: query='" + query + "' selected=" + selected.length + " popularTags=" + popLen + " tagsDb=" + dbKeys)

        var tagCounts = {}
        if (selected.length > 0) {
            var db = svc.tagsDb || {}
            for (var key in db) {
                var wTags = db[key]
                if (!wTags || !wTags.length) continue

                var hasAll = true
                for (var s = 0; s < selected.length; s++) {
                    if (wTags.indexOf(selected[s]) === -1) { hasAll = false; break }
                }
                if (!hasAll) continue
                for (var ti = 0; ti < wTags.length; ti++) {
                    var tag = wTags[ti]
                    tagCounts[tag] = (tagCounts[tag] || 0) + 1
                }
            }
        } else {
            var tags = svc.popularTags || []
            for (var pi = 0; pi < tags.length; pi++)
                tagCounts[tags[pi].tag] = tags[pi].count
        }

        var result = []
        var maxVisible = 60
        for (var tagName in tagCounts) {
            if (result.length >= maxVisible + selected.length) break
            var isSelected = selected.indexOf(tagName) !== -1
            var matchesSearch = !query || _fuzzyMatch(tagName, query)
            if (matchesSearch || isSelected)
                result.push({ tag: tagName, count: tagCounts[tagName], selected: isSelected })
        }
        result.sort(function(a, b) {
            if (a.selected !== b.selected) return a.selected ? -1 : 1
            return b.count - a.count
        })
        console.log("[TagCloud] _recomputeTags result: " + result.length + " tags to display")
        if (_entranceActive || _visibleTagsCache.length === 0) {
            _visibleTagsCache = result
        } else {
            _pendingTagsCache = result
            _tagsCrossfade.restart()
        }

        var suggest = ""
        if (query.length > 0) {
            var bestCount = -1
            for (var ai = 0; ai < result.length; ai++) {
                if (!result[ai].selected && (result[ai].tag.indexOf(query) === 0 || _stem(result[ai].tag) === _stem(query)) && result[ai].count > bestCount) {
                    suggest = result[ai].tag
                    bestCount = result[ai].count
                }
            }
        }
        _autoSuggestion = suggest
        _tagsDirty = false
    }

    Row {
        id: tagSearchRow
        anchors.top: parent.top
        anchors.topMargin: 8
        anchors.left: parent.left
        anchors.leftMargin: 14
        anchors.right: parent.right
        anchors.rightMargin: 14
        spacing: 8
        z: 12

        Text {
            text: "\u{f0349}"
            font.family: Style.fontFamilyNerdIcons
            font.pixelSize: 14 * Config.uiScale
            color: tagCloud.colors ? Qt.rgba(tagCloud.colors.surfaceText.r, tagCloud.colors.surfaceText.g, tagCloud.colors.surfaceText.b, 0.5) : Qt.rgba(1, 1, 1, 0.4)
            anchors.verticalCenter: parent.verticalCenter
        }

        Rectangle {
            id: tagSearchBox
            width: parent.width - 30 * Config.uiScale
            height: 26 * Config.uiScale
            radius: 13 * Config.uiScale
            color: tagCloud.colors ? Qt.rgba(tagCloud.colors.surface.r, tagCloud.colors.surface.g, tagCloud.colors.surface.b, 0.5) : Qt.rgba(0, 0, 0, 0.3)
            border.width: tagSearchInput.activeFocus ? 1 : 0
            border.color: tagCloud.colors ? Qt.rgba(tagCloud.colors.primary.r, tagCloud.colors.primary.g, tagCloud.colors.primary.b, 0.5) : Qt.rgba(1, 1, 1, 0.3)

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.IBeamCursor
                onClicked: tagSearchInput.forceActiveFocus()
            }

            TextInput {
                id: tagSearchInput
                anchors.fill: parent
                anchors.leftMargin: 12 * Config.uiScale
                anchors.rightMargin: 12 * Config.uiScale
                verticalAlignment: TextInput.AlignVCenter
                font.family: Style.fontFamily
                font.pixelSize: 11 * Config.uiScale
                font.letterSpacing: 0.3
                color: tagCloud.colors ? tagCloud.colors.surfaceText : "#fff"
                clip: true
                selectByMouse: true
                onTextChanged: {
                    if (tagCloud._syncingText) return
                    var raw = text.toLowerCase()
                    var words = raw.split(/\s+/).filter(function(w) { return w.length > 0 })
                    var svc = tagCloud.service
                    if (!svc) { tagCloud._tagSearchQuery = raw; return }

                    var allTags = svc.popularTags || []
                    var tagSet = {}
                    for (var i = 0; i < allTags.length; i++) tagSet[allTags[i].tag] = true

                    var locked = []
                    var partial = ""
                    for (var j = 0; j < words.length; j++) {
                        if (tagSet[words[j]]) locked.push(words[j])
                        else partial = words[j]
                    }

                    var endsWithSpace = raw.length > 0 && raw[raw.length - 1] === ' '
                    if (!endsWithSpace && words.length > 0) {
                        var lastWord = words[words.length - 1]
                        var li = locked.indexOf(lastWord)
                        if (li !== -1 && li === locked.length - 1) locked.pop()
                        partial = lastWord
                    }

                    var prev = svc.selectedTags
                    var changed = locked.length !== prev.length
                    if (!changed) {
                        for (var k = 0; k < locked.length; k++) {
                            if (prev.indexOf(locked[k]) === -1) { changed = true; break }
                        }
                        if (!changed) {
                            for (var m = 0; m < prev.length; m++) {
                                if (locked.indexOf(prev[m]) === -1) { changed = true; break }
                            }
                        }
                    }
                    if (changed) {
                        svc.selectedTags = locked
                        svc.updateFilteredModel(true)
                    }
                    tagCloud._tagSearchQuery = partial
                }
                Keys.onEscapePressed: {
                    if (text !== "") {
                        text = ""
                    } else {
                        tagCloud.closeRequested()
                    }
                }
                Keys.onDownPressed: function(event) {
                    if (event.modifiers & Qt.ShiftModifier) {
                        tagCloud.closeRequested()
                        event.accepted = true
                    } else {
                        event.accepted = false
                    }
                }
                Keys.onUpPressed: function(event) { event.accepted = false }
                Keys.onLeftPressed: function(event) {
                    if (event.modifiers & Qt.ShiftModifier) { event.accepted = false }
                }
                Keys.onRightPressed: function(event) {
                    if (event.modifiers & Qt.ShiftModifier) { event.accepted = false }
                }
                Keys.onTabPressed: function(event) {
                    event.accepted = true
                    var suggest = tagCloud._autoSuggestion
                    if (!suggest) return
                    var partial = tagCloud._tagSearchQuery
                    var raw = text.toLowerCase()
                    var lastIdx = raw.lastIndexOf(partial)
                    if (lastIdx !== -1)
                        tagSearchInput.text = text.substring(0, lastIdx) + suggest + " "
                }

                Text {
                    anchors.fill: parent
                    verticalAlignment: Text.AlignVCenter
                    text: "Search tags..."
                    font: parent.font
                    color: tagCloud.colors ? Qt.rgba(tagCloud.colors.surfaceText.r, tagCloud.colors.surfaceText.g, tagCloud.colors.surfaceText.b, 0.35) : Qt.rgba(1, 1, 1, 0.3)
                    visible: !parent.text && !parent.activeFocus
                }

                Text {
                    id: ghostText
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    verticalAlignment: Text.AlignVCenter
                    font: tagSearchInput.font
                    color: tagCloud.colors ? Qt.rgba(tagCloud.colors.surfaceText.r, tagCloud.colors.surfaceText.g, tagCloud.colors.surfaceText.b, 0.25) : Qt.rgba(1, 1, 1, 0.2)
                    visible: tagCloud._autoSuggestion.length > 0 && tagSearchInput.text.length > 0
                    text: {
                        if (!visible) return ""
                        var raw = tagSearchInput.text
                        var partial = tagCloud._tagSearchQuery
                        var suggest = tagCloud._autoSuggestion
                        if (!partial || !suggest) return ""
                        var lastIdx = raw.toLowerCase().lastIndexOf(partial)
                        if (lastIdx === -1) return ""
                        return raw.substring(0, lastIdx) + suggest
                    }
                }
            }

            Text {
                anchors.right: parent.right
                anchors.rightMargin: 8 * Config.uiScale
                anchors.verticalCenter: parent.verticalCenter
                text: "\u{f0156}"
                font.family: Style.fontFamilyNerdIcons
                font.pixelSize: 12 * Config.uiScale
                color: tagCloud.colors ? Qt.rgba(tagCloud.colors.surfaceText.r, tagCloud.colors.surfaceText.g, tagCloud.colors.surfaceText.b, 0.4) : Qt.rgba(1, 1, 1, 0.3)
                visible: tagSearchInput.text.length > 0

                MouseArea {
                    anchors.fill: parent
                    anchors.margins: -4
                    cursorShape: Qt.PointingHandCursor
                    onClicked: { tagSearchInput.text = ""; tagSearchInput.forceActiveFocus() }
                }
            }
        }
    }

    SequentialAnimation {
        id: _tagsCrossfade
        NumberAnimation { target: tagChipsArea; property: "opacity"; to: 0; duration: 80; easing.type: Easing.InQuad }
        ScriptAction { script: { tagCloud._visibleTagsCache = tagCloud._pendingTagsCache; tagCloud._pendingTagsCache = null } }
        NumberAnimation { target: tagChipsArea; property: "opacity"; to: 1; duration: 150; easing.type: Easing.OutCubic }
    }

    Item {
        id: tagChipsArea
        anchors.top: tagSearchRow.bottom
        anchors.topMargin: 6
        anchors.left: parent.left
        anchors.leftMargin: 10
        anchors.right: parent.right
        anchors.rightMargin: 10
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 6
        clip: true
        z: 11

        Flow {
            id: tagCloudFlow
            width: parent.width
            spacing: -4
            property var _service: tagCloud.service

            Repeater {
                model: tagCloud._visibleTagsCache

                Item {
                    id: tagParaChip
                    property bool isSelected: modelData.selected
                    property bool isHovered: tagParaMouse.containsMouse
                    property int skew: 10 * Config.uiScale
                    width: tagParaText.implicitWidth + 24 * Config.uiScale + skew
                    height: 24 * Config.uiScale
                    z: isSelected ? 10 : (isHovered ? 5 : 1)

                    property real _animProgress: 1
                    opacity: _animProgress
                    transform: Translate { y: (1 - tagParaChip._animProgress) * 12 }

                    Component.onCompleted: {
                        if (tagCloud._entranceActive) {
                            _animProgress = 0
                            _entranceAnim.start()
                        }
                    }

                    NumberAnimation {
                        id: _entranceAnim
                        target: tagParaChip
                        property: "_animProgress"
                        from: 0; to: 1
                        duration: 250
                        easing.type: Easing.OutCubic
                        Component.onCompleted: _entranceAnim.delay = index * 8
                    }

                    readonly property color _resolvedActiveColor: tagCloud.colors ? tagCloud.colors.primary : Style.fallbackAccent

                    Canvas {
                        id: tagCanvas
                        anchors.fill: parent
                        property color fillColor: tagParaChip.isSelected
                            ? tagParaChip._resolvedActiveColor
                            : (tagParaChip.isHovered
                                ? (tagCloud.colors ? Qt.rgba(tagCloud.colors.surfaceVariant.r, tagCloud.colors.surfaceVariant.g, tagCloud.colors.surfaceVariant.b, 0.6) : Qt.rgba(1, 1, 1, 0.15))
                                : (tagCloud.colors ? Qt.rgba(tagCloud.colors.surfaceContainer.r, tagCloud.colors.surfaceContainer.g, tagCloud.colors.surfaceContainer.b, 0.85) : Qt.rgba(0.1, 0.12, 0.18, 0.85)))
                        property color strokeColor: tagParaChip.isSelected
                            ? Qt.rgba(tagParaChip._resolvedActiveColor.r, tagParaChip._resolvedActiveColor.g, tagParaChip._resolvedActiveColor.b, 0.6)
                            : (tagCloud.colors ? Qt.rgba(tagCloud.colors.primary.r, tagCloud.colors.primary.g, tagCloud.colors.primary.b, 0.15) : Qt.rgba(1, 1, 1, 0.08))
                        onFillColorChanged: requestPaint()
                        onStrokeColorChanged: requestPaint()
                        onWidthChanged: requestPaint()
                        onPaint: {
                            var ctx = getContext("2d")
                            ctx.clearRect(0, 0, width, height)
                            var sk = tagParaChip.skew
                            ctx.fillStyle = fillColor
                            ctx.beginPath()
                            ctx.moveTo(sk, 0)
                            ctx.lineTo(width, 0)
                            ctx.lineTo(width - sk, height)
                            ctx.lineTo(0, height)
                            ctx.closePath()
                            ctx.fill()
                            ctx.strokeStyle = strokeColor
                            ctx.lineWidth = 1
                            ctx.stroke()
                        }
                    }

                    Text {
                        id: tagParaText
                        anchors.centerIn: parent
                        text: modelData.tag.toUpperCase()
                        color: tagParaChip.isSelected
                            ? (tagCloud.colors ? tagCloud.colors.primaryText : "#000")
                            : (tagCloud.colors ? tagCloud.colors.tertiary : "#8bceff")
                        font.family: Style.fontFamily
                        font.pixelSize: 10 * Config.uiScale
                        font.weight: tagParaChip.isSelected ? Font.Bold : Font.Bold
                        font.letterSpacing: 0.5
                    }

                    MouseArea {
                        id: tagParaMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            var svc = tagCloudFlow._service
                            if (!svc) return
                            var tag = modelData.tag
                            var tags = svc.selectedTags.slice()
                            var idx = tags.indexOf(tag)
                            var removing = idx !== -1
                            if (removing) tags.splice(idx, 1)
                            else tags.push(tag)
                            svc.selectedTags = tags
                            svc.updateFilteredModel(true)

                            tagCloud._syncingText = true
                            if (removing) {
                                var re = new RegExp('\\b' + tag + '\\b\\s*', 'i')
                                tagSearchInput.text = tagSearchInput.text.replace(re, '').replace(/^\s+/, '')
                            } else {
                                var cur = tagSearchInput.text
                                var suffix = (cur.length > 0 && cur[cur.length - 1] !== ' ') ? ' ' : ''
                                tagSearchInput.text = cur + suffix + tag + ' '
                            }
                            tagCloud._syncingText = false
                            tagCloud._recomputeTags()
                            tagSearchInput.forceActiveFocus()
                        }
                    }

                    StyledToolTip {
                        visible: tagParaMouse.containsMouse
                        text: modelData.tag + " (" + modelData.count + ")"
                        delay: 500
                    }
                }
            }

            property bool _settled: false

            populate: Transition { id: populateTransition }

            move: Transition {
                enabled: tagCloudFlow._settled
                NumberAnimation { properties: "x,y"; duration: Style.animNormal; easing.type: Easing.OutCubic }
            }

            Timer {
                id: settleTimer
                interval: 50
                onTriggered: tagCloudFlow._settled = true
            }

            onPositioningComplete: settleTimer.restart()
        }
    }
}
