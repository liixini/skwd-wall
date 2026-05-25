
import QtQuick
import QtQuick.Controls
import QtQuick.Shapes
import Quickshell.Io
import ".."
import "../services"

Item {
  id: settingsPanel

  property var colors
  property var service
  property bool settingsOpen: false
  property string activeTab: "selector"
  property bool openDownward: false

  property var _ollamaModels: []
  property bool _ollamaModelsFetching: false
  property string _ollamaFetchStdout: ""
  property string _lastConvertResult: ""
  property string _lastOptimizeResult: ""

  signal themeChanged(string scheme, string mode, var colorIndex)

  function _s(v) { return v * Config.uiScale }

  property var _ollamaFetchProc: Process {
    onExited: function(code) {
      settingsPanel._ollamaModelsFetching = false
      if (code === 0) {
        try {
          var resp = JSON.parse(settingsPanel._ollamaFetchStdout.trim())
          var names = (resp.models || []).map(function(m) { return m.name })
          names.sort()
          settingsPanel._ollamaModels = names
        } catch(e) { settingsPanel._ollamaModels = [] }
      } else { settingsPanel._ollamaModels = [] }
    }
    stdout: SplitParser {
      onRead: function(data) { settingsPanel._ollamaFetchStdout += data }
    }
  }

  function _fetchOllamaModels() {
    var url = Config.ollamaUrl || "http://localhost:11434"
    _ollamaModelsFetching = true
    _ollamaFetchStdout = ""
    _ollamaFetchProc.command = ["sh", "-c", "curl -s --max-time 5 '" + url + "/api/tags'"]
    _ollamaFetchProc.running = true
  }

  Connections {
    target: Config
    function onOllamaEnabledChanged() {
      if (!Config.ollamaEnabled && settingsPanel.activeTab === "ollama")
        settingsPanel.activeTab = "general"
    }
    function onMatugenEnabledChanged() {
      if (!Config.matugenEnabled && settingsPanel.activeTab === "matugen")
        settingsPanel.activeTab = "general"
    }
    function onSteamEnabledChanged() {
      if (!Config.steamEnabled && settingsPanel.activeTab === "steam")
        settingsPanel.activeTab = "general"
    }
    function onWallhavenEnabledChanged() {
      if (!Config.wallhavenEnabled && settingsPanel.activeTab === "wallhaven")
        settingsPanel.activeTab = "general"
    }
  }

  Connections {
    target: ImageOptimizeService
    function onFinished(optimized, skippedCount, failed) {
      var parts = []
      if (optimized > 0) parts.push(optimized + " optimized")
      if (skippedCount > 0) parts.push(skippedCount + " skipped")
      if (failed > 0) parts.push(failed + " failed")
      settingsPanel._lastOptimizeResult = parts.join(" · ") || "Nothing to optimize"
    }
  }

  z: 102
  width: (settingsPanel.activeTab === "performance" || settingsPanel.activeTab === "ollama" ? 920 : settingsPanel.activeTab === "general" ? 780 : 580) * Config.uiScale
  Behavior on width { NumberAnimation { duration: Style.animFast; easing.type: Easing.OutCubic } }
  height: tabRow.height + contentLoader.height + 36

  visible: settingsOpen
  opacity: settingsOpen ? 1 : 0
  scale: settingsOpen ? 1 : 0.9
  transformOrigin: openDownward ? Item.Top : Item.Bottom
  Behavior on opacity { NumberAnimation { duration: Style.animFast; easing.type: Easing.OutCubic } }
  Behavior on scale { NumberAnimation { duration: Style.animFast; easing.type: Easing.OutCubic } }

  signal closeRequested()

  Keys.onEscapePressed: closeRequested()
  focus: settingsOpen

  MouseArea {
    anchors.fill: parent
    acceptedButtons: Qt.LeftButton | Qt.RightButton
    onClicked: function(mouse) {
      if (mouse.button === Qt.RightButton) settingsPanel.closeRequested()
    }
  }

  function _cloneIntegrations() {
    return Config.integrations.map(function(e) { return JSON.parse(JSON.stringify(e)) })
  }

  function _saveField(key, value) {
    if (!Config._data.components || typeof Config._data.components.wallpaperSelector !== "object" || Config._data.components.wallpaperSelector === null)
      Config.saveKey("components.wallpaperSelector.enabled", true)
    Config.saveKey("components.wallpaperSelector." + key, value)
  }

  function _saveConfigKey(path, value) {
    Config.saveKey(path, value)
  }

  function _showWarning(title, message) {
    _warningPopup.title = title
    _warningPopup.message = message
    _warningPopup.open()
  }

  function _applyPreset(expanded, sliceH, sliceW, visible, gap, skew) {
    Config.saveKey("components.wallpaperSelector.expandedWidth", expanded)
    Config.saveKey("components.wallpaperSelector.sliceHeight", sliceH)
    Config.saveKey("components.wallpaperSelector.sliceWidth", sliceW)
    Config.saveKey("components.wallpaperSelector.visibleCount", visible)
    Config.saveKey("components.wallpaperSelector.sliceSpacing", gap)
    Config.saveKey("components.wallpaperSelector.skewOffset", skew)
  }

  function _saveCustomPreset(slot) {
    var key = slot + "_" + Config.displayMode
    var preset = {}
    if (Config.displayMode === "slices") {
      preset = {
        expandedWidth: Config.wallpaperExpandedWidth,
        sliceHeight: Config.wallpaperSliceHeight,
        sliceWidth: Config.wallpaperSliceWidth,
        visibleCount: Config.wallpaperVisibleCount,
        sliceSpacing: Config.wallpaperSliceSpacing,
        skewOffset: Config.wallpaperSkewOffset
      }
    } else if (Config.displayMode === "hex") {
      preset = {
        hexRadius: Config.hexRadius,
        hexRows: Config.hexRows,
        hexCols: Config.hexCols,
        hexScrollStep: Config.hexScrollStep,
        hexArc: Config.hexArc,
        hexArcIntensity: Config.hexArcIntensity
      }
    } else if (Config.displayMode === "wall") {
      preset = {
        gridColumns: Config.gridColumns,
        gridRows: Config.gridRows,
        gridThumbWidth: Config.gridThumbWidth,
        gridThumbHeight: Config.gridThumbHeight
      }
    }
    Config.saveKey("components.wallpaperSelector.customPresets." + key, preset)
  }

  function _loadCustomPreset(slot) {
    var key = slot + "_" + Config.displayMode
    var p = Config.wallpaperCustomPresets[key]
    if (!p) return
    if (Config.displayMode === "slices") {
      _applyPreset(p.expandedWidth, p.sliceHeight, p.sliceWidth, p.visibleCount, p.sliceSpacing, p.skewOffset)
    } else if (Config.displayMode === "hex") {
      if (p.hexRadius !== undefined) settingsPanel._saveField("hexRadius", p.hexRadius)
      if (p.hexRows !== undefined) settingsPanel._saveField("hexRows", p.hexRows)
      if (p.hexCols !== undefined) settingsPanel._saveField("hexCols", p.hexCols)
      if (p.hexScrollStep !== undefined) settingsPanel._saveField("hexScrollStep", p.hexScrollStep)
      if (p.hexArc !== undefined) settingsPanel._saveField("hexArc", p.hexArc)
      if (p.hexArcIntensity !== undefined) settingsPanel._saveField("hexArcIntensity", p.hexArcIntensity)
    } else if (Config.displayMode === "wall") {
      if (p.gridColumns !== undefined) settingsPanel._saveField("gridColumns", p.gridColumns)
      if (p.gridRows !== undefined) settingsPanel._saveField("gridRows", p.gridRows)
      if (p.gridThumbWidth !== undefined) settingsPanel._saveField("gridThumbWidth", p.gridThumbWidth)
      if (p.gridThumbHeight !== undefined) settingsPanel._saveField("gridThumbHeight", p.gridThumbHeight)
    }
  }

  property int _tabSkew: 14

  Row {
    id: tabRow
    anchors.horizontalCenter: parent.horizontalCenter
    anchors.top: parent.top
    anchors.topMargin: 12
    spacing: -settingsPanel._tabSkew
    z: 11

    add: Transition {
      NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Style.animNormal; easing.type: Easing.OutCubic }
      NumberAnimation { property: "scale"; from: 0.8; to: 1; duration: Style.animNormal; easing.type: Easing.OutCubic }
    }
    move: Transition {
      NumberAnimation { properties: "x"; duration: Style.animNormal; easing.type: Easing.OutCubic }
    }

    Repeater {
      model: {
        var tabs = [
          { key: "selector",  label: "SELECTOR" },
          { key: "paper",     label: "PAPER" },
          { key: "general",   label: "GENERAL" },
          { key: "paths",     label: "PATHS" },
          { key: "performance", label: "PERFORMANCE" },
          { key: "postprocessing", label: "EXTERNAL" },
          { key: "keybinds",  label: "KEYBINDS" },
          { key: "theme",     label: "THEME" }
        ]
        if (Config.wallhavenEnabled) tabs.push({ key: "wallhaven", label: "WALLHAVEN" })
        if (Config.steamEnabled) tabs.push({ key: "steam", label: "STEAM" })
        if (Config.ollamaEnabled) tabs.push({ key: "ollama", label: "OLLAMA" })
        if (Config.matugenEnabled) tabs.push({ key: "matugen", label: "MATUGEN" })
        if (Config.isNiri) tabs.push({ key: "niri", label: "NIRI" })
        return tabs
      }

      FilterButton {
        colors: settingsPanel.colors
        label: modelData.label
        skew: settingsPanel._tabSkew
        height: 28
        isActive: settingsPanel.activeTab === modelData.key
        onClicked: settingsPanel.activeTab = modelData.key
      }
    }
  }

  Item {
    id: contentLoader
    anchors.top: tabRow.bottom
    anchors.left: parent.left
    anchors.right: parent.right
    anchors.margins: 12
    anchors.topMargin: 8
    height: {
      if (settingsPanel.activeTab === "selector") return selectorContent.implicitHeight
      if (settingsPanel.activeTab === "paper") return paperContent.implicitHeight
      if (settingsPanel.activeTab === "general") return generalContent.implicitHeight
      if (settingsPanel.activeTab === "ollama") return ollamaContent.implicitHeight
      if (settingsPanel.activeTab === "paths") return pathsContent.implicitHeight
      if (settingsPanel.activeTab === "wallhaven") return wallhavenContent.implicitHeight
      if (settingsPanel.activeTab === "steam") return steamContent.implicitHeight
      if (settingsPanel.activeTab === "performance") return performanceContent.implicitHeight
      if (settingsPanel.activeTab === "postprocessing") return Math.min(postprocessingContent.implicitHeight, 360)
      if (settingsPanel.activeTab === "theme") return themeContent.implicitHeight
      if (settingsPanel.activeTab === "matugen") return Math.min(matugenContent.implicitHeight, 360)
      if (settingsPanel.activeTab === "keybinds") return keybindsContent.implicitHeight
      if (settingsPanel.activeTab === "niri") return niriContent.implicitHeight
      return 0
    }
    Behavior on height { NumberAnimation { duration: Style.animFast; easing.type: Easing.OutCubic } }


    Loader {
      id: selectorContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "selector"
      visible: active
      source: "settings/SelectorSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.saveField = function(k, v) { settingsPanel._saveField(k, v) }
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
        item.showWarning = function(t, m) { settingsPanel._showWarning(t, m) }
        item.applyPreset = function(ew, sh, sw, vc, gap, sk) { settingsPanel._applyPreset(ew, sh, sw, vc, gap, sk) }
        item.saveCustomPreset = function(slot) { settingsPanel._saveCustomPreset(slot) }
        item.loadCustomPreset = function(slot) { settingsPanel._loadCustomPreset(slot) }
      }
    }

    Loader {
      id: paperContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "paper"
      visible: active
      source: "settings/PaperSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
      }
    }

    Loader {
      id: generalContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "general"
      visible: active
      source: "settings/GeneralSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
      }
    }

    Loader {
      id: ollamaContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "ollama"
      visible: active
      source: "settings/OllamaSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.ollamaModels = Qt.binding(function() { return settingsPanel._ollamaModels })
        item.ollamaModelsFetching = Qt.binding(function() { return settingsPanel._ollamaModelsFetching })
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
        item.refreshModels = function() { settingsPanel._fetchOllamaModels() }
        item.openDeleteConfirm = function() { _deleteConfirmPopup.open() }
      }
    }

    Loader {
      id: pathsContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "paths"
      visible: active
      source: "settings/PathsSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
      }
    }

    Loader {
      id: niriContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "niri"
      visible: active
      source: "settings/NiriSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
      }
    }

    Loader {
      id: wallhavenContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "wallhaven"
      visible: active
      source: "settings/WallhavenSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.saveField = function(k, v) { settingsPanel._saveField(k, v) }
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
      }
    }

    Loader {
      id: steamContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "steam"
      visible: active
      source: "settings/SteamSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.saveField = function(k, v) { settingsPanel._saveField(k, v) }
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
      }
    }

    Loader {
      id: performanceContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "performance"
      visible: active
      source: "settings/PerformanceSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
        item.service = Qt.binding(function() { return settingsPanel.service })
        item.openOptimizeConfirm = function() { _optimizeConfirmPopup.open() }
      }
    }

    Loader {
      id: postprocessingContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "postprocessing"
      visible: active
      source: "settings/PostprocessingSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
      }
    }

    Loader {
      id: themeContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "theme"
      visible: active
      source: "settings/ThemeSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
        item.notifyThemeChanged = function(s, m, ci) { settingsPanel.themeChanged(s, m, ci) }
      }
    }

    Loader {
      id: matugenContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "matugen"
      visible: active
      source: "settings/MatugenSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
        item.saveConfigKey = function(k, v) { settingsPanel._saveConfigKey(k, v) }
        item.cloneIntegrations = function() { return settingsPanel._cloneIntegrations() }
      }
    }

    Loader {
      id: keybindsContent
      anchors.left: parent.left
      anchors.right: parent.right
      active: settingsPanel.activeTab === "keybinds"
      visible: active
      source: "settings/KeybindsSettings.qml"
      onLoaded: {
        item.colors = Qt.binding(function() { return settingsPanel.colors })
      }
    }
  }

  Rectangle {
    id: _deleteConfirmPopup
    visible: false
    anchors.fill: parent
    z: 200
    color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surface.r, settingsPanel.colors.surface.g, settingsPanel.colors.surface.b, 0.97) : Qt.rgba(0.08, 0.08, 0.12, 0.97)
    radius: 8

    function open() { _deleteConfirmInput.text = ""; visible = true; _deleteConfirmInput.forceActiveFocus() }
    function close() { visible = false }

    MouseArea { anchors.fill: parent; onClicked: function(mouse) { mouse.accepted = true } }

    Column {
      anchors.centerIn: parent
      spacing: 12
      width: parent.width * 0.7

      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: "\u{f0027}"
        font.family: Style.fontFamilyNerdIcons; font.pixelSize: settingsPanel._s(28)
        color: "#ef5350"
      }

      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: "DELETE ALL TAGS?"
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(14); font.weight: Font.Bold; font.letterSpacing: 1.5
        color: settingsPanel.colors ? settingsPanel.colors.surfaceText : "#fff"
      }

      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        text: "This will erase every tag and re-analyse all wallpapers with the current model. This cannot be undone."
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.letterSpacing: 0.2
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.6) : Qt.rgba(1, 1, 1, 0.5)
        wrapMode: Text.WordWrap
        lineHeight: 1.3
      }

      Item { width: 1; height: 2 }

      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: 'Type "delete" to confirm'
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11)
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.5) : Qt.rgba(1, 1, 1, 0.4)
      }

      Rectangle {
        anchors.horizontalCenter: parent.horizontalCenter
        width: 180; height: 30; radius: 15
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surface.r, settingsPanel.colors.surface.g, settingsPanel.colors.surface.b, 0.5) : Qt.rgba(0, 0, 0, 0.3)
        border.width: _deleteConfirmInput.activeFocus ? 1 : 0
        border.color: "#ef5350"

        TextInput {
          id: _deleteConfirmInput
          anchors.fill: parent
          anchors.leftMargin: 14; anchors.rightMargin: 14
          verticalAlignment: TextInput.AlignVCenter
          horizontalAlignment: TextInput.AlignHCenter
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(12); font.letterSpacing: 0.5
          color: settingsPanel.colors ? settingsPanel.colors.surfaceText : "#fff"
          clip: true
          Keys.onEscapePressed: _deleteConfirmPopup.close()
          Keys.onReturnPressed: {
            if (_deleteConfirmInput.text.toLowerCase().trim() === "delete") {
              WallpaperAnalysisService.regenerate()
              _deleteConfirmPopup.close()
            }
          }
        }
      }

      Row {
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: 8

        FilterButton {
          colors: settingsPanel.colors
          label: "CANCEL"
          skew: 8 * Config.uiScale; height: 26 * Config.uiScale
          onClicked: _deleteConfirmPopup.close()
        }

        FilterButton {
          id: _confirmDeleteBtn
          property bool canConfirm: _deleteConfirmInput.text.toLowerCase().trim() === "delete"
          colors: settingsPanel.colors
          label: "CONFIRM"
          skew: 8 * Config.uiScale; height: 26 * Config.uiScale
          hasActiveColor: true
          activeColor: canConfirm ? "#c62828" : Qt.rgba(0.5, 0.5, 0.5, 0.3)
          isActive: canConfirm
          activeOpacity: canConfirm ? 1.0 : 0.4
          onClicked: {
            if (canConfirm) {
              WallpaperAnalysisService.regenerate()
              _deleteConfirmPopup.close()
            }
          }
        }
      }
    }
  }

  Rectangle {
    id: _optimizeConfirmPopup
    visible: false
    anchors.fill: parent
    z: 201
    color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surface.r, settingsPanel.colors.surface.g, settingsPanel.colors.surface.b, 0.97) : Qt.rgba(0.08, 0.08, 0.12, 0.97)
    radius: 8

    function open() { visible = true }
    function close() { visible = false }

    MouseArea { anchors.fill: parent; onClicked: function(mouse) { mouse.accepted = true } }

    Column {
      anchors.centerIn: parent
      spacing: 12
      width: parent.width * 0.7

      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: "\u{f03e}"
        font.family: Style.fontFamilyNerdIcons; font.pixelSize: settingsPanel._s(28)
        color: settingsPanel.colors ? settingsPanel.colors.primary : Style.fallbackAccent
      }

      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: "OPTIMIZE ALL IMAGES?"
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(14); font.weight: Font.Bold; font.letterSpacing: 1.5
        color: settingsPanel.colors ? settingsPanel.colors.surfaceText : "#fff"
      }

      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        text: {
          var p = ImageOptimizeService.presets[Config.imageOptimizePreset]
          var r = ImageOptimizeService.resolutions[Config.imageOptimizeResolution]
          var fmts = p ? p.formats.join(", ").toUpperCase() : "?"
          return "This will convert " + fmts + " images to WebP using the " +
            Config.imageOptimizePreset.toUpperCase() + " preset (quality " + (p ? p.quality : "?") +
            ", max " + (r ? r.maxW + "x" + r.maxH : "?") +
            "). Originals are moved to trash. Already optimized files will be skipped."
        }
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.letterSpacing: 0.2
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.6) : Qt.rgba(1, 1, 1, 0.5)
        wrapMode: Text.WordWrap
        lineHeight: 1.3
      }

      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        text: "Only images in your wallpaper directory are processed"
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(10); font.letterSpacing: 0.2
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.4) : Qt.rgba(1, 1, 1, 0.35)
        wrapMode: Text.WordWrap
        lineHeight: 1.3
      }

      Item { width: 1; height: 4 }

      Row {
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: 8

        FilterButton {
          colors: settingsPanel.colors
          label: "CANCEL"
          skew: 8 * Config.uiScale; height: 26 * Config.uiScale
          onClicked: _optimizeConfirmPopup.close()
        }

        FilterButton {
          colors: settingsPanel.colors
          label: "OPTIMIZE"
          skew: 8 * Config.uiScale; height: 26 * Config.uiScale
          isActive: true
          onClicked: {
            _optimizeConfirmPopup.close()
            ImageOptimizeService.optimize(Config.imageOptimizePreset, Config.imageOptimizeResolution)
          }
        }
      }
    }
  }

  Rectangle {
    id: _convertConfirmPopup
    visible: false
    anchors.fill: parent
    z: 200
    color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surface.r, settingsPanel.colors.surface.g, settingsPanel.colors.surface.b, 0.97) : Qt.rgba(0.08, 0.08, 0.12, 0.97)
    radius: 8

    function open() { visible = true }
    function close() { visible = false }

    MouseArea { anchors.fill: parent; onClicked: function(mouse) { mouse.accepted = true } }

    Column {
      anchors.centerIn: parent
      spacing: 12
      width: parent.width * 0.7

      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: "\u{f03d}"
        font.family: Style.fontFamilyNerdIcons; font.pixelSize: settingsPanel._s(28)
        color: settingsPanel.colors ? settingsPanel.colors.primary : Style.fallbackAccent
      }

      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: "OPTIMIZE ALL VIDEOS?"
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(14); font.weight: Font.Bold; font.letterSpacing: 1.5
        color: settingsPanel.colors ? settingsPanel.colors.surfaceText : "#fff"
      }

      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        text: {
          var p = VideoConvertService.presets[Config.videoConvertPreset]
          var r = VideoConvertService.resolutions[Config.videoConvertResolution]
          return "This will convert all video wallpapers to HEVC (H.265) using the " +
            Config.videoConvertPreset.toUpperCase() + " preset (CRF " + (p ? p.crf : "?") +
            ", max " + (p ? p.maxrate : "?") + ", " + (r ? r.maxW + "x" + r.maxH : "?") +
            "). Originals are moved to trash. Already converted files will be skipped."
        }
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.letterSpacing: 0.2
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.6) : Qt.rgba(1, 1, 1, 0.5)
        wrapMode: Text.WordWrap
        lineHeight: 1.3
      }

      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        text: "This may take a while depending on the number and size of videos."
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(10); font.letterSpacing: 0.2
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.4) : Qt.rgba(1, 1, 1, 0.35)
        wrapMode: Text.WordWrap
        lineHeight: 1.3
      }

      Item { width: 1; height: 4 }

      Row {
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: 8

        FilterButton {
          colors: settingsPanel.colors
          label: "CANCEL"
          skew: 8 * Config.uiScale; height: 26 * Config.uiScale
          onClicked: _convertConfirmPopup.close()
        }

        FilterButton {
          colors: settingsPanel.colors
          label: "CONVERT"
          skew: 8 * Config.uiScale; height: 26 * Config.uiScale
          isActive: false
          enabled: false
          opacity: 0.35
        }
      }
    }
  }

  Rectangle {
    id: _warningPopup
    visible: false
    anchors.fill: parent
    z: 200
    color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surface.r, settingsPanel.colors.surface.g, settingsPanel.colors.surface.b, 0.97) : Qt.rgba(0.08, 0.08, 0.12, 0.97)
    radius: 8

    property string title: "RESTART REQUIRED"
    property string message: "Directory changes will take effect after restarting the app. Don't forget that includes the daemon!"

    function open() { visible = true }
    function close() { visible = false }

    MouseArea { anchors.fill: parent; onClicked: function(mouse) { mouse.accepted = true } }

    Column {
      anchors.centerIn: parent
      spacing: 12
      width: parent.width * 0.7

      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: "\u{f0028}"
        font.family: Style.fontFamilyNerdIcons; font.pixelSize: settingsPanel._s(28)
        color: "#ffb74d"
      }

      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: _warningPopup.title
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(14); font.weight: Font.Bold; font.letterSpacing: 1.5
        color: settingsPanel.colors ? settingsPanel.colors.surfaceText : "#fff"
      }

      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        text: _warningPopup.message
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.letterSpacing: 0.2
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.6) : Qt.rgba(1, 1, 1, 0.5)
        wrapMode: Text.WordWrap
        lineHeight: 1.3
      }

      Item { width: 1; height: 2 }

      FilterButton {
        anchors.horizontalCenter: parent.horizontalCenter
        colors: settingsPanel.colors
        label: "OK"
        skew: 8 * Config.uiScale; height: 26 * Config.uiScale
        isActive: true
        onClicked: _warningPopup.close()
      }
    }
  }
}
