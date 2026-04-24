
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

  signal themeChanged(string scheme, string mode)

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
  width: (settingsPanel.activeTab === "performance" || settingsPanel.activeTab === "general" ? 780 : 580) * Config.uiScale
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

  FileView {
    id: _selectorConfigFile
    path: Config.configDir + "/config.json"
    preload: true
  }

  function _readConfig() {
    _selectorConfigFile.reload()
    try { return JSON.parse(_selectorConfigFile.text()) } catch(e) { return {} }
  }

  function _cloneIntegrations() {
    return Config.integrations.map(function(e) { return JSON.parse(JSON.stringify(e)) })
  }

  function _saveField(key, value) {
    var data = _readConfig()
    if (!data.components) data.components = {}
    if (typeof data.components.wallpaperSelector !== "object" || data.components.wallpaperSelector === null)
      data.components.wallpaperSelector = { enabled: true }
    data.components.wallpaperSelector[key] = value
    _selectorConfigFile.setText(JSON.stringify(data, null, 2) + "\n")
  }

  function _saveConfigKey(path, value) {
    var data = _readConfig()
    var parts = path.split(".")
    var obj = data
    for (var i = 0; i < parts.length - 1; i++) {
      if (typeof obj[parts[i]] !== "object" || obj[parts[i]] === null)
        obj[parts[i]] = {}
      obj = obj[parts[i]]
    }
    obj[parts[parts.length - 1]] = value
    _selectorConfigFile.setText(JSON.stringify(data, null, 2) + "\n")
  }

  function _showWarning(title, message) {
    _warningPopup.title = title
    _warningPopup.message = message
    _warningPopup.open()
  }

  function _applyPreset(expanded, sliceH, sliceW, visible, gap, skew) {
    var data = _readConfig()
    if (!data.components) data.components = {}
    if (typeof data.components.wallpaperSelector !== "object" || data.components.wallpaperSelector === null)
      data.components.wallpaperSelector = { enabled: true }
    data.components.wallpaperSelector.expandedWidth = expanded
    data.components.wallpaperSelector.sliceHeight = sliceH
    data.components.wallpaperSelector.sliceWidth = sliceW
    data.components.wallpaperSelector.visibleCount = visible
    data.components.wallpaperSelector.sliceSpacing = gap
    data.components.wallpaperSelector.skewOffset = skew
    _selectorConfigFile.setText(JSON.stringify(data, null, 2) + "\n")
  }

  function _saveCustomPreset(slot) {
    var data = _readConfig()
    if (!data.components) data.components = {}
    if (typeof data.components.wallpaperSelector !== "object" || data.components.wallpaperSelector === null)
      data.components.wallpaperSelector = { enabled: true }
    if (!data.components.wallpaperSelector.customPresets)
      data.components.wallpaperSelector.customPresets = {}
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
    data.components.wallpaperSelector.customPresets[key] = preset
    _selectorConfigFile.setText(JSON.stringify(data, null, 2) + "\n")
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
          { key: "general",   label: "GENERAL" },
          { key: "paths",     label: "PATHS" },
          { key: "performance", label: "PERFORMANCE" },
          { key: "postprocessing", label: "POSTPROCESSING" },
          { key: "keybinds",  label: "KEYBINDS" },
          { key: "theme",     label: "THEME" }
        ]
        if (Config.wallhavenEnabled) tabs.push({ key: "wallhaven", label: "WALLHAVEN" })
        if (Config.steamEnabled) tabs.push({ key: "steam", label: "STEAM" })
        if (Config.ollamaEnabled) tabs.push({ key: "ollama", label: "OLLAMA" })
        if (Config.matugenEnabled) tabs.push({ key: "matugen", label: "MATUGEN" })
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
      if (settingsPanel.activeTab === "general") return generalContent.implicitHeight
      if (settingsPanel.activeTab === "ollama") return ollamaContent.implicitHeight
      if (settingsPanel.activeTab === "paths") return pathsContent.implicitHeight
      if (settingsPanel.activeTab === "wallhaven") return wallhavenContent.implicitHeight
      if (settingsPanel.activeTab === "steam") return steamContent.implicitHeight
      if (settingsPanel.activeTab === "performance") return performanceContent.implicitHeight
      if (settingsPanel.activeTab === "postprocessing") return Math.min(_postprocessingInner.implicitHeight, 360)
      if (settingsPanel.activeTab === "theme") return _themeInner.implicitHeight
      if (settingsPanel.activeTab === "matugen") return Math.min(_matugenInner.implicitHeight, 360)
      if (settingsPanel.activeTab === "keybinds") return keybindsContent.implicitHeight
      return 0
    }
    Behavior on height { NumberAnimation { duration: Style.animFast; easing.type: Easing.OutCubic } }

    Rectangle {
      anchors.fill: parent
      anchors.margins: -8
      radius: 6
      color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surface.r, settingsPanel.colors.surface.g, settingsPanel.colors.surface.b, 0.5) : Qt.rgba(0, 0, 0, 0.3)
    }

    Flow {
      id: selectorContent
      anchors.left: parent.left
      anchors.right: parent.right
      visible: settingsPanel.activeTab === "selector"
      spacing: 12

      readonly property bool _slicesMode: Config.displayMode === "slices"
      readonly property int _segCount: _slicesMode ? 4 : 3
      readonly property real _availW: width - spacing * (2 * _segCount - 2) - (_segCount - 1)
      readonly property real _w1: _availW * 0.38
      readonly property real _w2: _availW * (_slicesMode ? 0.22 : 0.31)
      readonly property real _w3: _availW * (_slicesMode ? 0.22 : 0.31)
      readonly property real _w4: _availW * 0.18

      Column {
        width: parent._w1
        spacing: 8

        Text {
          text: "LAYOUT"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Row {
          width: parent.width; spacing: -4
          Repeater {
            model: [
              { key: "slices",  label: "Slices" },
              { key: "hex",     label: "Hex" },
              { key: "wall",    label: "Wall" },
              { key: "mosaic",  label: "Mosaic" }
            ]
            FilterButton {
              colors: settingsPanel.colors
              label: modelData.label
              skew: 8 * Config.uiScale; height: 26 * Config.uiScale
              isActive: Config.displayMode === modelData.key
              onClicked: {
                if (modelData.key === "mosaic" && Config.displayMode !== "mosaic")
                  settingsPanel._showWarning("MOSAIC IS EXPERIMENTAL", "Not all features work yet. Please do not expect everything to function correctly.")
                settingsPanel._saveField("displayMode", modelData.key)
              }
            }
          }
        }

        Item { width: 1; height: 2 }

        Text {
          text: "PRESETS"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Row {
          width: parent.width; spacing: -4
          visible: Config.displayMode === "slices"
          Repeater {
            model: [
              { label: "XS", expanded: 360,  sliceH: 200, sliceW: 52,  visible: 20, gap: -30, skew: 16 },
              { label: "S",  expanded: 480,  sliceH: 270, sliceW: 68,  visible: 18, gap: -30, skew: 20 },
              { label: "M",  expanded: 768,  sliceH: 432, sliceW: 108, visible: 14, gap: -30, skew: 28 },
              { label: "L",  expanded: 924,  sliceH: 520, sliceW: 135, visible: 12, gap: -30, skew: 35 },
              { label: "XL", expanded: 1280, sliceH: 720, sliceW: 180, visible: 9,  gap: -30, skew: 45 }
            ]
            FilterButton {
              colors: settingsPanel.colors
              label: modelData.label
              skew: 8 * Config.uiScale; height: 26 * Config.uiScale
              isActive: Config.wallpaperExpandedWidth === modelData.expanded && Config.wallpaperSliceHeight === modelData.sliceH
              onClicked: settingsPanel._applyPreset(modelData.expanded, modelData.sliceH, modelData.sliceW, modelData.visible, modelData.gap, modelData.skew)
              tooltip: modelData.expanded + "×" + modelData.sliceH + " (16:9)"
            }
          }
        }

        Row {
          width: parent.width; spacing: -4
          Repeater {
            model: ["C1", "C2", "C3", "C4"]
            FilterButton {
              property string presetKey: modelData + "_" + Config.displayMode
              property var presetData: Config.wallpaperCustomPresets[presetKey] || null
              property bool isEmpty: !presetData
              colors: settingsPanel.colors
              label: modelData
              skew: 8 * Config.uiScale; height: 26 * Config.uiScale
              isActive: {
                if (isEmpty) return false
                if (Config.displayMode === "slices") return Config.wallpaperExpandedWidth === presetData.expandedWidth && Config.wallpaperSliceHeight === presetData.sliceHeight
                if (Config.displayMode === "hex") return Config.hexRadius === presetData.hexRadius && Config.hexRows === presetData.hexRows && Config.hexCols === presetData.hexCols
                if (Config.displayMode === "wall") return Config.gridColumns === presetData.gridColumns && Config.gridRows === presetData.gridRows
                return false
              }
              activeOpacity: isEmpty ? 0.35 : 1.0
              tooltip: {
                if (isEmpty) return "Click to save current"
                if (Config.displayMode === "slices") return presetData.expandedWidth + "×" + presetData.sliceHeight + " - Right-click to overwrite"
                if (Config.displayMode === "hex") return "r" + presetData.hexRadius + " " + presetData.hexRows + "×" + presetData.hexCols + " - Right-click to overwrite"
                if (Config.displayMode === "wall") return presetData.gridColumns + "×" + presetData.gridRows + " " + presetData.gridThumbWidth + "×" + presetData.gridThumbHeight + " - Right-click to overwrite"
                return ""
              }
              onClicked: {
                if (isEmpty) settingsPanel._saveCustomPreset(modelData)
                else settingsPanel._loadCustomPreset(modelData)
              }
              MouseArea {
                anchors.fill: parent; acceptedButtons: Qt.RightButton
                cursorShape: Qt.PointingHandCursor
                onClicked: settingsPanel._saveCustomPreset(modelData)
              }
            }
          }
        }
      }

      Rectangle {
        width: 0; height: 0; visible: false
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.1) : Qt.rgba(1, 1, 1, 0.08)
      }

      Column {
        width: parent._w2
        spacing: 6

        Text {
          text: Config.displayMode === "hex" ? "HEX GRID" : (Config.displayMode === "wall" ? "WALL" : (Config.displayMode === "mosaic" ? "MOSAIC" : "SIZE"))
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsInput { visible: Config.displayMode === "slices"; colors: settingsPanel.colors; label: "Height"; value: Config.wallpaperSliceHeight; min: 200; max: 1200; onCommit: function(n) { settingsPanel._saveField("sliceHeight", n) } }
        SettingsInput { visible: Config.displayMode === "slices"; colors: settingsPanel.colors; label: "Visible items"; value: Config.wallpaperVisibleCount; min: 3; max: 30; onCommit: function(n) { settingsPanel._saveField("visibleCount", n) } }
        SettingsInput { visible: Config.displayMode === "slices"; colors: settingsPanel.colors; label: "Selected width"; value: Config.wallpaperExpandedWidth; min: 50; max: 1800; onCommit: function(n) { settingsPanel._saveField("expandedWidth", n) } }
        SettingsInput { visible: Config.displayMode === "hex"; colors: settingsPanel.colors; label: "Radius"; value: Config.hexRadius; min: 60; max: 300; onCommit: function(n) { settingsPanel._saveField("hexRadius", n) } }
        SettingsInput { visible: Config.displayMode === "hex"; colors: settingsPanel.colors; label: "Rows"; value: Config.hexRows; min: 1; max: 8; onCommit: function(n) { settingsPanel._saveField("hexRows", n) } }
        SettingsInput { visible: Config.displayMode === "hex"; colors: settingsPanel.colors; label: "Columns"; value: Config.hexCols; min: 3; max: 20; onCommit: function(n) { settingsPanel._saveField("hexCols", n) } }
        SettingsInput { visible: Config.displayMode === "hex"; colors: settingsPanel.colors; label: "Scroll step"; value: Config.hexScrollStep; min: 1; max: 10; onCommit: function(n) { settingsPanel._saveField("hexScrollStep", n) } }
        SettingsToggle { visible: Config.displayMode === "hex"; colors: settingsPanel.colors; label: "Arc layout"; checked: Config.hexArc; onToggle: function(v) { settingsPanel._saveField("hexArc", v) } }
        SettingsInput { visible: Config.displayMode === "hex" && Config.hexArc; colors: settingsPanel.colors; label: "Arc intensity (×10)"; value: Math.round(Config.hexArcIntensity * 10); min: 1; max: 30; onCommit: function(n) { settingsPanel._saveField("hexArcIntensity", n / 10) } }
        SettingsInput { visible: Config.displayMode === "wall"; colors: settingsPanel.colors; label: "Columns"; value: Config.gridColumns; min: 2; max: 12; onCommit: function(n) { settingsPanel._saveField("gridColumns", n) } }
        SettingsInput { visible: Config.displayMode === "wall"; colors: settingsPanel.colors; label: "Rows"; value: Config.gridRows; min: 1; max: 8; onCommit: function(n) { settingsPanel._saveField("gridRows", n) } }
        SettingsInput { visible: Config.displayMode === "wall"; colors: settingsPanel.colors; label: "Thumb width"; value: Config.gridThumbWidth; min: 100; max: 600; onCommit: function(n) { settingsPanel._saveField("gridThumbWidth", n) } }
        SettingsInput { visible: Config.displayMode === "wall"; colors: settingsPanel.colors; label: "Thumb height"; value: Config.gridThumbHeight; min: 50; max: 400; onCommit: function(n) { settingsPanel._saveField("gridThumbHeight", n) } }

        SettingsInput { visible: Config.displayMode === "mosaic"; colors: settingsPanel.colors; label: "Cells"; value: Config.mosaicCells; min: 4; max: 200; onCommit: function(n) { settingsPanel._saveField("mosaicCells", n) } }
        SettingsInput { visible: Config.displayMode === "mosaic"; colors: settingsPanel.colors; label: "Seed"; value: Config.mosaicSeed; min: 1; max: 99999; onCommit: function(n) { settingsPanel._saveField("mosaicSeed", n) } }
        SettingsInput { visible: Config.displayMode === "mosaic"; colors: settingsPanel.colors; label: "Relax iterations"; value: Config.mosaicRelaxation; min: 0; max: 8; onCommit: function(n) { settingsPanel._saveField("mosaicRelaxation", n) } }
        SettingsInput { visible: Config.displayMode === "mosaic"; colors: settingsPanel.colors; label: "Width"; value: Config.mosaicWidth; min: 400; max: 3000; onCommit: function(n) { settingsPanel._saveField("mosaicWidth", n) } }
        SettingsInput { visible: Config.displayMode === "mosaic"; colors: settingsPanel.colors; label: "Height"; value: Config.mosaicHeight; min: 200; max: 2000; onCommit: function(n) { settingsPanel._saveField("mosaicHeight", n) } }
      }

      Rectangle {
        width: 0; height: 0; visible: false
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.1) : Qt.rgba(1, 1, 1, 0.08)
      }

      Column {
        width: parent._w3
        spacing: 6

        Text {
          text: "GEOMETRY"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
          visible: Config.displayMode === "slices"
        }

        SettingsInput { visible: Config.displayMode === "slices"; colors: settingsPanel.colors; label: "Slice width"; value: Config.wallpaperSliceWidth; min: 50; max: 500; onCommit: function(n) { settingsPanel._saveField("sliceWidth", n) } }
        SettingsInput { visible: Config.displayMode === "slices"; colors: settingsPanel.colors; label: "Gap"; value: Config.wallpaperSliceSpacing; min: -500; max: 500; onCommit: function(n) { settingsPanel._saveField("sliceSpacing", n) } }
        SettingsInput { visible: Config.displayMode === "slices"; colors: settingsPanel.colors; label: "Skew"; value: Config.wallpaperSkewOffset; min: -500; max: 500; onCommit: function(n) { settingsPanel._saveField("skewOffset", n) } }

        Item { width: 1; height: Config.displayMode === "slices" ? 4 : 0; visible: Config.displayMode === "slices" }

        Text {
          text: "APPLY"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Per monitor"
          checked: Config.wallpaperPerMonitor
          onToggle: function(v) { settingsPanel._saveConfigKey("general.wallpaperPerMonitor", v) }
        }

        Text {
          width: parent.width
          text: "WIP — Video and WE support coming."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(10)
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.4) : Qt.rgba(1, 1, 1, 0.3)
          wrapMode: Text.Wrap
        }
      }

      Rectangle {
        visible: false
        width: 0; height: 0
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.1) : Qt.rgba(1, 1, 1, 0.08)
      }

      Column {
        visible: Config.displayMode === "slices"
        width: parent._w4
        spacing: 6

        Text {
          text: "CORNERS"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Round corners"
          checked: Config.wallpaperSliceRoundCorners
          onToggle: function(v) { settingsPanel._saveField("roundCorners", v) }
        }

        SettingsInput {
          visible: Config.wallpaperSliceRoundCorners
          colors: settingsPanel.colors
          label: "Radius"
          value: Config.wallpaperSliceCornerRadius
          min: 0
          max: 80
          onCommit: function(n) { settingsPanel._saveField("cornerRadius", n) }
        }
      }
    }

    Flow {
      id: generalContent
      anchors.left: parent.left
      anchors.right: parent.right
      visible: settingsPanel.activeTab === "general"
      spacing: 12

      Column {
        width: (parent.width - 36) / 4
        spacing: 6

        Text {
          text: "GENERAL"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "Monitor"
          value: Config.mainMonitor
          placeholder: "e.g. DP-1"
          onCommit: function(v) { settingsPanel._saveConfigKey("monitor", v) }
        }

        SettingsCombo {
          colors: settingsPanel.colors
          label: "Color source"
          value: Config.colorSource
          model: ["ollama", "magick"]
          onSelect: function(v) { settingsPanel._saveConfigKey("colorSource", v) }
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "Locale (weather filter)"
          value: Config.locale
          placeholder: "e.g. London"
          onCommit: function(v) { settingsPanel._saveConfigKey("general.locale", v) }
        }

      }

      Column {
        width: (parent.width - 36) / 4
        spacing: 6

        Text {
          text: "FEATURES"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Matugen (Colour theming)"
          checked: Config.matugenEnabled
          onToggle: function(v) { settingsPanel._saveConfigKey("features.matugen", v) }
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Ollama (Automated tagging)"
          checked: Config.ollamaEnabled
          onToggle: function(v) { settingsPanel._saveConfigKey("features.ollama", v) }
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Steam Workshop browser"
          checked: Config.steamEnabled
          onToggle: function(v) { settingsPanel._saveConfigKey("features.steam", v) }
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Wallhaven browser"
          checked: Config.wallhavenEnabled
          onToggle: function(v) { settingsPanel._saveConfigKey("features.wallhaven", v) }
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Mute wallpaper audio"
          checked: Config.wallpaperMute
          onToggle: function(v) {
            settingsPanel._saveConfigKey("wallpaperMute", v)
            _muteReloadTimer.restart()
          }
        }

        SettingsSlider {
          colors: settingsPanel.colors
          label: "Wallpaper volume"
          value: Config.wallpaperVolume
          min: 0
          max: 100
          enabled: !Config.wallpaperMute
          onCommit: function(v) {
            settingsPanel._saveConfigKey("wallpaperVolume", v)
            _muteReloadTimer.restart()
          }
        }
      }

      Timer {
        id: _muteReloadTimer
        interval: 500
        repeat: false
        onTriggered: DaemonClient.restore()
      }

      Column {
        width: (parent.width - 36) / 4
        spacing: 6

        Text {
          text: "MORE FEATURES"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Close on selection"
          checked: Config.closeOnSelection
          onToggle: function(v) { settingsPanel._saveConfigKey("general.closeOnSelection", v) }
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Reopen at last selection"
          checked: Config.reopenAtLastSelection
          onToggle: function(v) { settingsPanel._saveConfigKey("general.reopenAtLastSelection", v) }
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Always show filter bar"
          checked: Config.filterBarAlwaysVisible
          onToggle: function(v) { settingsPanel._saveConfigKey("general.filterBarAlwaysVisible", v) }
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Always show search bar"
          checked: Config.searchBarAlwaysVisible
          onToggle: function(v) { settingsPanel._saveConfigKey("general.searchBarAlwaysVisible", v) }
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Video auto scale"
          checked: Config.videoAutoScale
          onToggle: function(v) { settingsPanel._saveConfigKey("features.videoAutoScale", v) }
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Notify on wallpaper change"
          checked: Config.notifyOnWallpaperChange
          onToggle: function(v) { settingsPanel._saveConfigKey("general.notifyOnWallpaperChange", v) }
        }

        Text {
          text: "BUILT-IN NOTIFICATIONS"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.weight: Font.Bold; font.letterSpacing: 1.2
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.tertiary.r, settingsPanel.colors.tertiary.g, settingsPanel.colors.tertiary.b, 0.8) : Qt.rgba(1, 1, 1, 0.4)
          topPadding: 6
        }

        Row {
          width: parent.width; spacing: -4
          Repeater {
            model: [
              { key: "auto",   label: "Auto" },
              { key: "always", label: "Always" },
              { key: "never",  label: "Never" }
            ]
            FilterButton {
              colors: settingsPanel.colors
              label: modelData.label
              skew: 8 * Config.uiScale; height: 24 * Config.uiScale
              isActive: Config.notificationsBuiltIn === modelData.key
              tooltip: {
                if (modelData.key === "auto") return "Run skwd's notification daemon only if no other one is active"
                if (modelData.key === "always") return "Always run skwd's notification daemon (may conflict with mako/dunst/etc.)"
                return "Never run skwd's notification daemon (use your own)"
              }
              onClicked: settingsPanel._saveConfigKey("notifications.builtIn", modelData.key)
            }
          }
        }

        Text {
          width: parent.width
          text: "Restart daemon to apply"
          wrapMode: Text.WordWrap
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(9); font.italic: true
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.outline.r, settingsPanel.colors.outline.g, settingsPanel.colors.outline.b, 0.7) : Qt.rgba(1, 1, 1, 0.3)
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "UI scale (1.0–2.0)"
          value: Config.uiScale.toFixed(2)
          placeholder: "1.00"
          onCommit: function(v) {
            var n = parseFloat(v)
            if (isNaN(n)) n = 1.0
            n = Math.max(1.0, Math.min(2.0, n))
            settingsPanel._saveConfigKey("general.uiScale", n)
          }
        }
      }

      Column {
        width: (parent.width - 36) / 4
        spacing: 6

        Text {
          text: "RANDOM"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsInput {
          colors: settingsPanel.colors
          label: "Rotation interval (seconds)"
          value: Config.randomInterval
          min: 1
          max: 86400
          onCommit: function(v) { settingsPanel._saveConfigKey("general.randomInterval", v) }
        }

        Text {
          text: "SOURCES"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.weight: Font.Bold; font.letterSpacing: 1.2
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.tertiary.r, settingsPanel.colors.tertiary.g, settingsPanel.colors.tertiary.b, 0.8) : Qt.rgba(1, 1, 1, 0.4)
          topPadding: 4
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Images"
          checked: Config.randomIncludeStatic
          onToggle: function(v) {
            if (!v && !Config.randomIncludeVideo && !Config.randomIncludeWE) return
            settingsPanel._saveConfigKey("general.randomIncludeStatic", v)
          }
        }
        SettingsToggle {
          colors: settingsPanel.colors
          label: "Video"
          checked: Config.randomIncludeVideo
          onToggle: function(v) {
            if (!v && !Config.randomIncludeStatic && !Config.randomIncludeWE) return
            settingsPanel._saveConfigKey("general.randomIncludeVideo", v)
          }
        }
        SettingsToggle {
          colors: settingsPanel.colors
          label: "WE"
          checked: Config.randomIncludeWE
          onToggle: function(v) {
            if (!v && !Config.randomIncludeStatic && !Config.randomIncludeVideo) return
            settingsPanel._saveConfigKey("general.randomIncludeWE", v)
          }
        }
        SettingsToggle {
          colors: settingsPanel.colors
          label: "Favourites"
          checked: Config.randomIncludeFavourites
          onToggle: function(v) {
            settingsPanel._saveConfigKey("general.randomIncludeFavourites", v)
          }
        }

      }
    }

    Flow {
      id: ollamaContent
      anchors.left: parent.left
      anchors.right: parent.right
      visible: settingsPanel.activeTab === "ollama"
      spacing: 12

      onVisibleChanged: {
        if (visible) settingsPanel._fetchOllamaModels()
      }

      Column {
        width: (parent.width - 12) / 2
        spacing: 6

        Text {
          text: "CONNECTION"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "URL"
          value: Config.ollamaUrl
          placeholder: "http://localhost:11434"
          onCommit: function(v) {
            settingsPanel._saveConfigKey("ollama.url", v)
            settingsPanel._fetchOllamaModels()
          }
        }

        SettingsCombo {
          colors: settingsPanel.colors
          label: settingsPanel._ollamaModelsFetching ? "Model  󰔟" : (settingsPanel._ollamaModels.length === 0 ? "Model  (no models found)" : "Model")
          model: settingsPanel._ollamaModels
          value: Config.ollamaModel
          onSelect: function(v) { settingsPanel._saveConfigKey("ollama.model", v) }
        }

        SettingsCombo {
          colors: settingsPanel.colors
          label: settingsPanel._ollamaModelsFetching ? "Consolidation Model  󰔟" : (settingsPanel._ollamaModels.length === 0 ? "Consolidation Model  (no models found)" : "Consolidation Model")
          model: settingsPanel._ollamaModels
          value: Config.ollamaConsolidationModel
          onSelect: function(v) { settingsPanel._saveConfigKey("ollama.consolidationModel", v) }
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "WIP! Tag consolidation Extremely alpha"
          checked: Config.ollamaConsolidateEnabled
          onToggle: function(v) { settingsPanel._saveConfigKey("ollama.consolidateEnabled", v) }
        }

        FilterButton {
          colors: settingsPanel.colors
          icon: "󰑐"
          tooltip: "Refresh model list"
          onClicked: settingsPanel._fetchOllamaModels()
        }
      }

      Rectangle {
        width: 0; height: 0; visible: false
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.1) : Qt.rgba(1, 1, 1, 0.08)
      }

      Column {
        width: (parent.width - 12) / 2
        spacing: 6

        Text {
          text: "DATA"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Item {
          width: parent.width; height: 28

          FilterButton {
            id: _consolidateTagsBtn
            colors: settingsPanel.colors
            label: "CONSOLIDATE TAGS"
            skew: 8 * Config.uiScale; height: 26 * Config.uiScale
            hasActiveColor: true
            isActive: _consolidateTagsBtn.isHovered
            enabled: Config.ollamaConsolidateEnabled
            opacity: enabled ? 1.0 : 0.35
            onClicked: WallpaperAnalysisService.consolidate()
          }
        }

        Text {
          width: parent.width
          text: "Sends all existing tags to Ollama to merge synonyms into canonical forms."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(10); font.letterSpacing: 0.2
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.45) : Qt.rgba(1, 1, 1, 0.35)
          wrapMode: Text.WordWrap
          lineHeight: 1.3
        }

        Item {
          width: parent.width; height: 28

          FilterButton {
            id: _deleteTagsBtn
            colors: settingsPanel.colors
            label: "DELETE ALL TAGS"
            skew: 8 * Config.uiScale; height: 26 * Config.uiScale
            hasActiveColor: true
            activeColor: "#c62828"
            isActive: _deleteTagsBtn.isHovered
            onClicked: _deleteConfirmPopup.open()
          }
        }

        Text {
          width: parent.width
          text: "Clears all Ollama-generated tags. The next analysis pass will re-tag everything with the current model."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(10); font.letterSpacing: 0.2
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.45) : Qt.rgba(1, 1, 1, 0.35)
          wrapMode: Text.WordWrap
          lineHeight: 1.3
        }
      }
    }

    Flow {
      id: pathsContent
      anchors.left: parent.left
      anchors.right: parent.right
      visible: settingsPanel.activeTab === "paths"
      spacing: 12

      Column {
        width: (parent.width - parent.spacing * 2 - 1) * 0.5
        spacing: 6

        Text {
          text: "DIRECTORIES"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "Wallpaper directory"
          value: Config.wallpaperDir
          placeholder: "~/Pictures/Wallpapers"
          onFocused: function() { settingsPanel._showWarning("RESTART REQUIRED", "Directory changes will take effect after restarting the app. Don't forget that includes the daemon!") }
          onCommit: function(v) { settingsPanel._saveConfigKey("paths.wallpaper", v) }
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "Video directory"
          value: Config.videoDir
          placeholder: "(same as wallpaper directory)"
          onFocused: function() { settingsPanel._showWarning("RESTART REQUIRED", "Directory changes will take effect after restarting the app. Don't forget that includes the daemon!") }
          onCommit: function(v) { settingsPanel._saveConfigKey("paths.videoWallpaper", v) }
        }
      }

      Rectangle {
        width: 0; height: 0; visible: false
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.1) : Qt.rgba(1, 1, 1, 0.08)
      }

      Column {
        width: (parent.width - parent.spacing * 2 - 1) * 0.5
        spacing: 6

        Text {
          text: "STEAM"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "Workshop directory"
          value: Config.weDir
          placeholder: "Steam Workshop content path"
          onCommit: function(v) { settingsPanel._saveConfigKey("paths.steamWorkshop", v) }
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "WE assets directory"
          value: Config.weAssetsDir
          placeholder: "Wallpaper Engine assets path"
          onCommit: function(v) { settingsPanel._saveConfigKey("paths.steamWeAssets", v) }
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "Steam directory"
          value: Config.steamDir
          placeholder: "Steam install path"
          onCommit: function(v) { settingsPanel._saveConfigKey("paths.steam", v) }
        }
      }
    }

    Flow {
      id: wallhavenContent
      anchors.left: parent.left
      anchors.right: parent.right
      visible: settingsPanel.activeTab === "wallhaven"
      spacing: 12

      Column {
        width: (parent.width - parent.spacing * 2 - 1) * 0.5
        spacing: 6

        Text {
          text: "GRID"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsInput {
          colors: settingsPanel.colors
          label: "Columns"
          value: Config.wallhavenColumns
          min: 2; max: 12
          onCommit: function(n) { settingsPanel._saveField("wallhavenColumns", n) }
        }

        SettingsInput {
          colors: settingsPanel.colors
          label: "Rows"
          value: Config.wallhavenRows
          min: 1; max: 10
          onCommit: function(n) { settingsPanel._saveField("wallhavenRows", n) }
        }

        Text {
          text: "THUMBNAIL"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
          topPadding: 8
        }

        SettingsInput {
          colors: settingsPanel.colors
          label: "Width"
          value: Config.wallhavenThumbWidth
          min: 100; max: 600
          onCommit: function(n) { settingsPanel._saveField("wallhavenThumbWidth", n) }
        }

        SettingsInput {
          colors: settingsPanel.colors
          label: "Height"
          value: Config.wallhavenThumbHeight
          min: 60; max: 600
          onCommit: function(n) { settingsPanel._saveField("wallhavenThumbHeight", n) }
        }
      }

      Rectangle { width: 1; height: parent.height; color: Qt.rgba(1, 1, 1, 0.08) }

      Column {
        width: (parent.width - parent.spacing * 2 - 1) * 0.5
        spacing: 6

        Text {
          text: "API"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "API key"
          value: Config.wallhavenApiKey
          placeholder: "Wallhaven API key (for NSFW)"
          onCommit: function(v) { settingsPanel._saveConfigKey("wallhaven.apiKey", v) }
        }
      }
    }

    Flow {
      id: steamContent
      anchors.left: parent.left
      anchors.right: parent.right
      visible: settingsPanel.activeTab === "steam"
      spacing: 12

      Column {
        width: (parent.width - parent.spacing * 2 - 1) * 0.5
        spacing: 6

        Text {
          text: "GRID"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsInput {
          colors: settingsPanel.colors
          label: "Columns"
          value: Config.steamColumns
          min: 2; max: 12
          onCommit: function(n) { settingsPanel._saveField("steamColumns", n) }
        }

        SettingsInput {
          colors: settingsPanel.colors
          label: "Rows"
          value: Config.steamRows
          min: 1; max: 10
          onCommit: function(n) { settingsPanel._saveField("steamRows", n) }
        }

        Text {
          text: "THUMBNAIL"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
          topPadding: 8
        }

        SettingsInput {
          colors: settingsPanel.colors
          label: "Width"
          value: Config.steamThumbWidth
          min: 100; max: 600
          onCommit: function(n) { settingsPanel._saveField("steamThumbWidth", n) }
        }

        SettingsInput {
          colors: settingsPanel.colors
          label: "Height"
          value: Config.steamThumbHeight
          min: 60; max: 600
          onCommit: function(n) { settingsPanel._saveField("steamThumbHeight", n) }
        }
      }

      Rectangle { width: 1; height: parent.height; color: Qt.rgba(1, 1, 1, 0.08) }

      Column {
        width: (parent.width - parent.spacing * 2 - 1) * 0.5
        spacing: 6

        Text {
          text: "API"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "API key"
          value: Config.steamApiKey
          placeholder: "Steam API key"
          onCommit: function(v) { settingsPanel._saveConfigKey("steam.apiKey", v) }
        }

        SettingsTextInput {
          colors: settingsPanel.colors
          label: "Username"
          value: Config.steamUsername
          placeholder: "Steam username (for steamcmd)"
          onCommit: function(v) { settingsPanel._saveConfigKey("steam.username", v) }
        }
      }
    }

    Flow {
      id: performanceContent
      anchors.left: parent.left
      anchors.right: parent.right
      visible: settingsPanel.activeTab === "performance"
      spacing: 12

      Column {
        width: (parent.width - parent.spacing * 6 - 3) / 4
        spacing: 6

        Text {
          text: "IMAGE OPTIMIZATION"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Text {
          width: parent.width
          text: "Converts PNG, JPEG, and GIF images to WebP format. Smaller file sizes with no visible quality loss. Steam Workshop assets are never modified."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.letterSpacing: 0.2
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceVariantText.r, settingsPanel.colors.surfaceVariantText.g, settingsPanel.colors.surfaceVariantText.b, 0.8) : Qt.rgba(1, 1, 1, 0.5)
          wrapMode: Text.WordWrap
          lineHeight: 1.3
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Auto-optimize new images"
          checked: Config.autoOptimizeImages
          onToggle: function(v) { settingsPanel._saveConfigKey("performance.autoOptimizeImages", v) }
        }

        SettingsCombo {
          colors: settingsPanel.colors
          label: "Quality"
          model: ["light", "balanced", "quality"]
          value: Config.imageOptimizePreset
          onSelect: function(v) { settingsPanel._saveConfigKey("performance.imageOptimizePreset", v) }
        }

        Repeater {
          model: [
            { key: "light",    desc: "Q 82 · max compression" },
            { key: "balanced", desc: "Q 88 · good trade-off" },
            { key: "quality",  desc: "Q 94 · visually lossless" }
          ]
          Text {
            text: (Config.imageOptimizePreset === modelData.key ? "▸ " : "  ") + modelData.key.toUpperCase() + ":  " + modelData.desc
            font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(10); font.letterSpacing: 0.2
            color: Config.imageOptimizePreset === modelData.key
              ? (settingsPanel.colors ? settingsPanel.colors.primary : Style.fallbackAccent)
              : (settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceVariantText.r, settingsPanel.colors.surfaceVariantText.g, settingsPanel.colors.surfaceVariantText.b, 0.7) : Qt.rgba(1, 1, 1, 0.4))
          }
        }

        SettingsCombo {
          colors: settingsPanel.colors
          label: "Max resolution"
          model: ["1080p", "2k", "4k"]
          value: Config.imageOptimizeResolution
          onSelect: function(v) { settingsPanel._saveConfigKey("performance.imageOptimizeResolution", v) }
        }

        Text {
          width: parent.width
          text: "Images above the cap are downscaled. Smaller images are never upscaled."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.letterSpacing: 0.2
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceVariantText.r, settingsPanel.colors.surfaceVariantText.g, settingsPanel.colors.surfaceVariantText.b, 0.8) : Qt.rgba(1, 1, 1, 0.5)
          wrapMode: Text.WordWrap
          lineHeight: 1.3
        }

        Item { width: 1; height: 2 }

        Row {
          spacing: 8

          FilterButton {
            colors: settingsPanel.colors
            label: ImageOptimizeService.running ? "CANCEL" : "OPTIMIZE ALL"
            skew: 8
            height: 28
            isActive: ImageOptimizeService.running
            onClicked: {
              if (ImageOptimizeService.running) ImageOptimizeService.cancel()
              else _optimizeConfirmPopup.open()
            }
          }

          Text {
            anchors.verticalCenter: parent.verticalCenter
            visible: !ImageOptimizeService.running && settingsPanel._lastOptimizeResult !== ""
            text: settingsPanel._lastOptimizeResult
            font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(10); font.letterSpacing: 0.2
            color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceVariantText.r, settingsPanel.colors.surfaceVariantText.g, settingsPanel.colors.surfaceVariantText.b, 0.8) : Qt.rgba(1, 1, 1, 0.5)
          }
        }
      }

      Rectangle {
        width: 0; height: 0; visible: false
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.1) : Qt.rgba(1, 1, 1, 0.08)
      }

      Item {
        width: (parent.width - parent.spacing * 6 - 3) / 4
        height: _videoOptCol.implicitHeight

        Column {
          id: _videoOptCol
          width: parent.width
          spacing: 6
          opacity: 0.35
          enabled: false

          Text {
            text: "VIDEO OPTIMIZATION  ·  WIP"
            font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
            color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
          }

          Text {
            width: parent.width
            text: "Re-encodes video wallpapers to HEVC (H.265) for significantly smaller sizes. This feature is currently under development."
            font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.letterSpacing: 0.2
            color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceVariantText.r, settingsPanel.colors.surfaceVariantText.g, settingsPanel.colors.surfaceVariantText.b, 0.8) : Qt.rgba(1, 1, 1, 0.5)
            wrapMode: Text.WordWrap
            lineHeight: 1.3
          }

          SettingsToggle {
            colors: settingsPanel.colors
            label: "Auto-convert new videos"
            checked: false
          }

          SettingsCombo {
            colors: settingsPanel.colors
            label: "Quality"
            model: ["light", "balanced", "quality"]
            value: Config.videoConvertPreset
          }

          Repeater {
            model: [
              { key: "light",    desc: "CRF 28 · 6 Mbps" },
              { key: "balanced", desc: "CRF 26 · 10 Mbps" },
              { key: "quality",  desc: "CRF 23 · 16 Mbps" }
            ]
            Text {
              text: (Config.videoConvertPreset === modelData.key ? "▸ " : "  ") + modelData.key.toUpperCase() + ":  " + modelData.desc
              font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(10); font.letterSpacing: 0.2
              color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceVariantText.r, settingsPanel.colors.surfaceVariantText.g, settingsPanel.colors.surfaceVariantText.b, 0.7) : Qt.rgba(1, 1, 1, 0.4)
            }
          }

          SettingsCombo {
            colors: settingsPanel.colors
            label: "Max resolution"
            model: ["1080p", "2k", "4k"]
            value: Config.videoConvertResolution
          }

          Item { width: 1; height: 2 }

          Row {
            spacing: 8

            FilterButton {
              colors: settingsPanel.colors
              label: "OPTIMIZE ALL"
              skew: 8
              height: 28
            }
          }
        }
      }

      Rectangle {
        width: 0; height: 0; visible: false
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.1) : Qt.rgba(1, 1, 1, 0.08)
      }

      Column {
        width: (parent.width - parent.spacing * 6 - 3) / 4
        spacing: 6

        Text {
          text: "VIDEO PREVIEWS"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Text {
          width: parent.width
          text: "Play animated thumbnails when hovering over video wallpapers."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.letterSpacing: 0.2
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceVariantText.r, settingsPanel.colors.surfaceVariantText.g, settingsPanel.colors.surfaceVariantText.b, 0.8) : Qt.rgba(1, 1, 1, 0.5)
          wrapMode: Text.WordWrap
          lineHeight: 1.3
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Video previews"
          checked: Config.videoPreviewEnabled
          onToggle: function(v) { settingsPanel._saveConfigKey("features.videoPreview", v) }
        }

        Item { width: 1; height: 8 }

        Text {
          text: "TRASH"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Text {
          width: parent.width
          text: "Originals are moved to trash before optimization, so you can recover them if needed."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.letterSpacing: 0.2
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceVariantText.r, settingsPanel.colors.surfaceVariantText.g, settingsPanel.colors.surfaceVariantText.b, 0.8) : Qt.rgba(1, 1, 1, 0.5)
          wrapMode: Text.WordWrap
          lineHeight: 1.3
        }

        Item { width: 1; height: 2 }

        Text {
          text: "IMAGES"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.weight: Font.Bold; font.letterSpacing: 1.2
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsInput {
          colors: settingsPanel.colors
          label: "Retention (days)"
          value: Config.imageTrashDays
          min: 1; max: 365
          onCommit: function(v) { settingsPanel._saveConfigKey("performance.imageTrashDays", v) }
        }

        SettingsToggle {
          colors: settingsPanel.colors
          label: "Auto-delete after retention"
          checked: Config.autoDeleteImageTrash
          onToggle: function(v) { settingsPanel._saveConfigKey("performance.autoDeleteImageTrash", v) }
        }

        Item { width: 1; height: 4 }

        Item {
          width: parent.width
          height: _videoTrashCol.implicitHeight
          opacity: 0.35
          enabled: false

          Column {
            id: _videoTrashCol
            width: parent.width
            spacing: 6

            Text {
              text: "VIDEOS  ·  WIP"
              font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.weight: Font.Bold; font.letterSpacing: 1.2
              color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
            }

            SettingsInput {
              colors: settingsPanel.colors
              label: "Retention (days)"
              value: Config.videoTrashDays
              min: 1; max: 365
            }

            SettingsToggle {
              colors: settingsPanel.colors
              label: "Auto-delete after retention"
              checked: false
            }
          }
        }
      }

      Rectangle {
        width: 0; height: 0; visible: false
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.1) : Qt.rgba(1, 1, 1, 0.08)
      }

      Column {
        width: (parent.width - parent.spacing * 6 - 3) / 4
        spacing: 6

        Text {
          text: "THUMBNAILS"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Text {
          width: parent.width
          text: "Maximum number of thumbnail jobs that run in parallel during cache rebuilds."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.letterSpacing: 0.2
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceVariantText.r, settingsPanel.colors.surfaceVariantText.g, settingsPanel.colors.surfaceVariantText.b, 0.8) : Qt.rgba(1, 1, 1, 0.5)
          wrapMode: Text.WordWrap
          lineHeight: 1.3
        }

        SettingsInput {
          colors: settingsPanel.colors
          label: "Max concurrent jobs"
          value: Config.maxThumbJobs
          min: 1; max: 64
          onCommit: function(v) { settingsPanel._saveConfigKey("performance.maxThumbJobs", v) }
        }

        Item { width: 1; height: 8 }

        Text {
          text: "CACHE"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Text {
          width: parent.width
          text: "Clear all cached thumbnails and regenerate from scratch."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.letterSpacing: 0.2
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceVariantText.r, settingsPanel.colors.surfaceVariantText.g, settingsPanel.colors.surfaceVariantText.b, 0.8) : Qt.rgba(1, 1, 1, 0.5)
          wrapMode: Text.WordWrap
          lineHeight: 1.3
        }

        Item { width: 1; height: 2 }

        Row {
          spacing: 8

          FilterButton {
            colors: settingsPanel.colors
            label: DaemonClient.cacheRunning ? "CLEARING..." : "CLEAR ALL DATA"
            skew: 8
            height: 28
            enabled: !DaemonClient.cacheRunning
            onClicked: settingsPanel.service.clearData()
          }
        }
      }
    }

    Flickable {
      id: postprocessingContent
      anchors.left: parent.left
      anchors.right: parent.right
      height: parent.height
      visible: settingsPanel.activeTab === "postprocessing"
      contentHeight: _postprocessingInner.implicitHeight
      clip: true
      flickableDirection: Flickable.VerticalFlick
      boundsBehavior: Flickable.StopAtBounds

      ScrollBar.vertical: ScrollBar {
        policy: ScrollBar.AlwaysOff
      }

      function _snapshotCmds() {
        var cmds = []
        for (var i = 0; i < postCmdRepeater.count; i++) {
          var item = postCmdRepeater.itemAt(i)
          if (item) cmds.push(item.children[0].children[0].text)
        }
        return cmds
      }

      Column {
        id: _postprocessingInner
        width: parent.width
        spacing: 8

      SettingsToggle {
        colors: settingsPanel.colors
        label: "Run on startup restore"
        checked: Config.postProcessOnRestore
        onToggle: function(v) { settingsPanel._saveConfigKey("postProcessOnRestore", v) }
      }

      Text {
        text: "COMMANDS"
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
        color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
      }

      Text {
        width: parent.width
        text: "Shell commands to run after every wallpaper change. Use %type% (static/video/we), %name%, and %path% as placeholders."
        font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11)
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.6) : Qt.rgba(1, 1, 1, 0.4)
        wrapMode: Text.Wrap
      }

      Rectangle {
        width: 120; height: 28; radius: 4
        color: addMa.containsMouse
          ? (settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.2) : Qt.rgba(1, 1, 1, 0.15))
          : (settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceContainer.r, settingsPanel.colors.surfaceContainer.g, settingsPanel.colors.surfaceContainer.b, 0.6) : Qt.rgba(0.15, 0.15, 0.2, 0.6))

        Text {
          anchors.centerIn: parent
          text: "+ ADD COMMAND"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.weight: Font.Bold; font.letterSpacing: 0.5
          color: settingsPanel.colors ? settingsPanel.colors.primary : Style.fallbackAccent
        }

        MouseArea {
          id: addMa
          anchors.fill: parent
          hoverEnabled: true
          cursorShape: Qt.PointingHandCursor
          onClicked: {
            var cmds = postprocessingContent._snapshotCmds()
            cmds.push("")
            settingsPanel._saveConfigKey("postProcessing", cmds)
          }
        }
      }

      Repeater {
        id: postCmdRepeater
        model: Config.postProcessing

        Row {
          width: _postprocessingInner.width
          spacing: 6

          Rectangle {
            width: parent.width - removeBtn.width - parent.spacing
            height: 26
            radius: 4
            color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceContainer.r, settingsPanel.colors.surfaceContainer.g, settingsPanel.colors.surfaceContainer.b, 0.6) : Qt.rgba(0.15, 0.15, 0.2, 0.6)
            border.width: cmdInput.activeFocus ? 1 : 0
            border.color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.5) : Qt.rgba(1, 1, 1, 0.3)

            TextInput {
              id: cmdInput
              anchors.fill: parent
              anchors.leftMargin: 8
              anchors.rightMargin: 8
              verticalAlignment: TextInput.AlignVCenter
              font.family: Style.fontFamilyCode
              font.pixelSize: settingsPanel._s(11)
              color: settingsPanel.colors ? settingsPanel.colors.tertiary : "#8bceff"
              clip: true
              selectByMouse: true
              text: modelData

              onEditingFinished: {
                var cmds = postprocessingContent._snapshotCmds()
                settingsPanel._saveConfigKey("postProcessing", cmds)
              }
            }
          }

          Rectangle {
            id: removeBtn
            width: 26; height: 26; radius: 4
            color: removeMa.containsMouse
              ? (settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.25) : Qt.rgba(1, 0.3, 0.3, 0.25))
              : "transparent"

            Text {
              anchors.centerIn: parent
              text: "✕"
              font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold
              color: settingsPanel.colors ? settingsPanel.colors.primary : Qt.rgba(1, 0.3, 0.3, 0.8)
            }

            MouseArea {
              id: removeMa
              anchors.fill: parent
              hoverEnabled: true
              cursorShape: Qt.PointingHandCursor
              onClicked: {
                var cmds = postprocessingContent._snapshotCmds()
                cmds.splice(index, 1)
                settingsPanel._saveConfigKey("postProcessing", cmds)
              }
            }
          }
        }
      }

    }
    }

    Rectangle {
      id: _postScrollTrack
      x: postprocessingContent.x - 6
      width: 3
      radius: 1.5
      opacity: 0.5
      visible: postprocessingContent.visible
      color: settingsPanel.colors ? settingsPanel.colors.primary : Qt.rgba(1, 1, 1, 0.6)
      readonly property real _cH: postprocessingContent.contentHeight
      readonly property real _vH: postprocessingContent.height
      readonly property bool _overflow: _cH > _vH && _cH > 0
      height: _overflow ? Math.min(_vH * 0.5, Math.max(16, _vH * _vH / _cH)) : 0
      y: postprocessingContent.y + (_overflow
        ? postprocessingContent.contentY / (_cH - _vH) * (_vH - height)
        : 0)
      Behavior on height { NumberAnimation { duration: 150 } }
    }

    Flickable {
      id: themeContent
      anchors.left: parent.left
      anchors.right: parent.right
      height: parent.height
      visible: settingsPanel.activeTab === "theme"
      contentHeight: _themeInner.implicitHeight
      clip: true
      flickableDirection: Flickable.VerticalFlick
      boundsBehavior: Flickable.StopAtBounds

      ScrollBar.vertical: ScrollBar {
        policy: ScrollBar.AlwaysOff
      }

      Column {
        id: _themeInner
        width: parent.width
        spacing: 8

        Text {
          text: "SCHEME TYPE"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsCombo {
          colors: settingsPanel.colors
          label: ""
          value: Config.matugenScheme.replace("scheme-", "")
          model: ["content", "expressive", "fidelity", "fruit-salad", "monochrome", "neutral", "rainbow", "tonal-spot", "vibrant"]
          onSelect: function(v) { var full = "scheme-" + v; settingsPanel._saveConfigKey("matugen.schemeType", full); settingsPanel.themeChanged(full, Config.matugenMode) }
        }

        Text {
          text: "MODE"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        SettingsCombo {
          colors: settingsPanel.colors
          label: ""
          value: Config.matugenMode
          model: ["dark", "light"]
          onSelect: function(v) { settingsPanel._saveConfigKey("matugen.mode", v); settingsPanel.themeChanged(Config.matugenScheme, v) }
        }
      }
    }

    Flickable {
      id: matugenContent
      anchors.left: parent.left
      anchors.right: parent.right
      height: parent.height
      visible: settingsPanel.activeTab === "matugen"
      contentHeight: _matugenInner.implicitHeight
      clip: true
      flickableDirection: Flickable.VerticalFlick
      boundsBehavior: Flickable.StopAtBounds

      ScrollBar.vertical: ScrollBar {
        policy: ScrollBar.AlwaysOff
      }

      Column {
        id: _matugenInner
        width: parent.width
        spacing: 8

        Text {
          text: "EXTERNAL MATUGEN CONFIG"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Text {
          width: parent.width
          text: "Path to an external matugen config file such as the one from your existing setup. This runs alongside Skwd-wall's internal Matugen configuration."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(10)
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.5) : Qt.rgba(1, 1, 1, 0.35)
          wrapMode: Text.Wrap
        }

        Rectangle {
          width: parent.width; height: 26; radius: 4
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceContainer.r, settingsPanel.colors.surfaceContainer.g, settingsPanel.colors.surfaceContainer.b, 0.6) : Qt.rgba(0.15, 0.15, 0.2, 0.6)
          border.width: _defaultCfgInput.activeFocus ? 1 : 0
          border.color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.5) : Qt.rgba(1, 1, 1, 0.3)

          TextInput {
            id: _defaultCfgInput
            anchors.fill: parent; anchors.leftMargin: 8; anchors.rightMargin: 8
            verticalAlignment: TextInput.AlignVCenter
            font.family: Style.fontFamilyCode; font.pixelSize: settingsPanel._s(11)
            color: settingsPanel.colors ? settingsPanel.colors.tertiary : "#8bceff"
            clip: true; selectByMouse: true
            text: Config.defaultMatugenConfig
            onEditingFinished: settingsPanel._saveConfigKey("defaultMatugenConfig", text)
          }
        }

        Item { width: 1; height: 2 }

        Text {
          text: "EXTERNAL MATUGEN COMMAND"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Text {
          width: parent.width
          text: "Shell command to run with the external config. Use %config% for the config path and %path% for the wallpaper path. Matugen v4 users: add --source-color-index 0 after 'image' to avoid interactive prompts."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(10)
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.5) : Qt.rgba(1, 1, 1, 0.35)
          wrapMode: Text.Wrap
        }

        Rectangle {
          width: parent.width; height: 26; radius: 4
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceContainer.r, settingsPanel.colors.surfaceContainer.g, settingsPanel.colors.surfaceContainer.b, 0.6) : Qt.rgba(0.15, 0.15, 0.2, 0.6)
          border.width: _extMatugenCmdInput.activeFocus ? 1 : 0
          border.color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.5) : Qt.rgba(1, 1, 1, 0.3)

          TextInput {
            id: _extMatugenCmdInput
            anchors.fill: parent; anchors.leftMargin: 8; anchors.rightMargin: 8
            verticalAlignment: TextInput.AlignVCenter
            font.family: Style.fontFamilyCode; font.pixelSize: settingsPanel._s(11)
            color: settingsPanel.colors ? settingsPanel.colors.tertiary : "#8bceff"
            clip: true; selectByMouse: true
            text: Config.externalMatugenCommand
            onEditingFinished: settingsPanel._saveConfigKey("externalMatugenCommand", text)
          }
        }

        Text {
          text: "INTEGRATIONS"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Text {
          width: parent.width
          text: "Matugen colour-theming integrations. Each entry generates themed output from a template and optionally runs a reload command."
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11)
          color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceText.r, settingsPanel.colors.surfaceText.g, settingsPanel.colors.surfaceText.b, 0.6) : Qt.rgba(1, 1, 1, 0.4)
          wrapMode: Text.Wrap
        }

        Repeater {
          model: Config.integrations

          Rectangle {
            width: _matugenInner.width
            height: _integRow.implicitHeight + 12
            radius: 4
            color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceContainer.r, settingsPanel.colors.surfaceContainer.g, settingsPanel.colors.surfaceContainer.b, 0.4) : Qt.rgba(0.15, 0.15, 0.2, 0.4)

            Row {
              id: _integRow
              anchors.left: parent.left; anchors.right: parent.right
              anchors.margins: 6; anchors.verticalCenter: parent.verticalCenter
              spacing: 6

              Column {
                width: (parent.width - _integRemoveBtn.width - parent.spacing * 2) * 0.2
                spacing: 2
                Text { text: "name"; font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(9); color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1,1,1,0.4) }
                Rectangle {
                  width: parent.width; height: 22; radius: 3
                  color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceContainer.r, settingsPanel.colors.surfaceContainer.g, settingsPanel.colors.surfaceContainer.b, 0.6) : Qt.rgba(0.15,0.15,0.2,0.6)
                  border.width: _nameIn.activeFocus ? 1 : 0
                  border.color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.5) : Qt.rgba(1,1,1,0.3)
                  TextInput {
                    id: _nameIn; anchors.fill: parent; anchors.leftMargin: 4; anchors.rightMargin: 4
                    verticalAlignment: TextInput.AlignVCenter; font.family: Style.fontFamilyCode; font.pixelSize: settingsPanel._s(10)
                    color: settingsPanel.colors ? settingsPanel.colors.surfaceText : "#ccc"; clip: true; selectByMouse: true
                    text: modelData.name || ""
                    onEditingFinished: { var a = settingsPanel._cloneIntegrations(); a[index].name = text; settingsPanel._saveConfigKey("integrations", a) }
                  }
                }
              }

              Column {
                width: (parent.width - _integRemoveBtn.width - parent.spacing * 2) * 0.25
                spacing: 2
                Text { text: "template"; font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(9); color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1,1,1,0.4) }
                Rectangle {
                  width: parent.width; height: 22; radius: 3
                  color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceContainer.r, settingsPanel.colors.surfaceContainer.g, settingsPanel.colors.surfaceContainer.b, 0.6) : Qt.rgba(0.15,0.15,0.2,0.6)
                  border.width: _tplIn.activeFocus ? 1 : 0
                  border.color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.5) : Qt.rgba(1,1,1,0.3)
                  TextInput {
                    id: _tplIn; anchors.fill: parent; anchors.leftMargin: 4; anchors.rightMargin: 4
                    verticalAlignment: TextInput.AlignVCenter; font.family: Style.fontFamilyCode; font.pixelSize: settingsPanel._s(10)
                    color: settingsPanel.colors ? settingsPanel.colors.tertiary : "#8bceff"; clip: true; selectByMouse: true
                    text: modelData.template || ""
                    onEditingFinished: { var a = settingsPanel._cloneIntegrations(); a[index].template = text; settingsPanel._saveConfigKey("integrations", a) }
                  }
                }
              }

              Column {
                width: (parent.width - _integRemoveBtn.width - parent.spacing * 2) * 0.3
                spacing: 2
                Text { text: "output"; font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(9); color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1,1,1,0.4) }
                Rectangle {
                  width: parent.width; height: 22; radius: 3
                  color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceContainer.r, settingsPanel.colors.surfaceContainer.g, settingsPanel.colors.surfaceContainer.b, 0.6) : Qt.rgba(0.15,0.15,0.2,0.6)
                  border.width: _outIn.activeFocus ? 1 : 0
                  border.color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.5) : Qt.rgba(1,1,1,0.3)
                  TextInput {
                    id: _outIn; anchors.fill: parent; anchors.leftMargin: 4; anchors.rightMargin: 4
                    verticalAlignment: TextInput.AlignVCenter; font.family: Style.fontFamilyCode; font.pixelSize: settingsPanel._s(10)
                    color: settingsPanel.colors ? settingsPanel.colors.tertiary : "#8bceff"; clip: true; selectByMouse: true
                    text: modelData.output || ""
                    onEditingFinished: { var a = settingsPanel._cloneIntegrations(); a[index].output = text; settingsPanel._saveConfigKey("integrations", a) }
                  }
                }
              }

              Column {
                width: (parent.width - _integRemoveBtn.width - parent.spacing * 2) * 0.25
                spacing: 2
                Text { text: "reload"; font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(9); color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1,1,1,0.4) }
                Rectangle {
                  width: parent.width; height: 22; radius: 3
                  color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceContainer.r, settingsPanel.colors.surfaceContainer.g, settingsPanel.colors.surfaceContainer.b, 0.6) : Qt.rgba(0.15,0.15,0.2,0.6)
                  border.width: _reloadIn.activeFocus ? 1 : 0
                  border.color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.5) : Qt.rgba(1,1,1,0.3)
                  TextInput {
                    id: _reloadIn; anchors.fill: parent; anchors.leftMargin: 4; anchors.rightMargin: 4
                    verticalAlignment: TextInput.AlignVCenter; font.family: Style.fontFamilyCode; font.pixelSize: settingsPanel._s(10)
                    color: settingsPanel.colors ? settingsPanel.colors.tertiary : "#8bceff"; clip: true; selectByMouse: true
                    text: modelData.reload || ""
                    onEditingFinished: { var a = settingsPanel._cloneIntegrations(); a[index].reload = text || undefined; settingsPanel._saveConfigKey("integrations", a) }
                  }
                }
              }

              Rectangle {
                id: _integRemoveBtn
                width: 22; height: 22; radius: 3
                anchors.verticalCenter: parent.verticalCenter
                color: _integRemoveMa.containsMouse
                  ? (settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.25) : Qt.rgba(1,0.3,0.3,0.25))
                  : "transparent"
                Text {
                  anchors.centerIn: parent; text: "✕"
                  font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.weight: Font.Bold
                  color: settingsPanel.colors ? settingsPanel.colors.primary : Qt.rgba(1,0.3,0.3,0.8)
                }
                MouseArea {
                  id: _integRemoveMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                  onClicked: { var a = settingsPanel._cloneIntegrations(); a.splice(index, 1); settingsPanel._saveConfigKey("integrations", a) }
                }
              }
            }
          }
        }

        Rectangle {
          width: 150; height: 28; radius: 4
          color: _addIntegMa.containsMouse
            ? (settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.2) : Qt.rgba(1,1,1,0.15))
            : (settingsPanel.colors ? Qt.rgba(settingsPanel.colors.surfaceContainer.r, settingsPanel.colors.surfaceContainer.g, settingsPanel.colors.surfaceContainer.b, 0.6) : Qt.rgba(0.15,0.15,0.2,0.6))
          Text {
            anchors.centerIn: parent; text: "+ ADD INTEGRATION"
            font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.weight: Font.Bold; font.letterSpacing: 0.5
            color: settingsPanel.colors ? settingsPanel.colors.primary : Style.fallbackAccent
          }
          MouseArea {
            id: _addIntegMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
            onClicked: { var a = settingsPanel._cloneIntegrations(); a.push({name: "", template: "", output: ""}); settingsPanel._saveConfigKey("integrations", a) }
          }
        }
      }
    }

    Rectangle {
      id: _matugenScrollTrack
      x: matugenContent.x - 6
      width: 3
      radius: 1.5
      opacity: 0.5
      visible: matugenContent.visible
      color: settingsPanel.colors ? settingsPanel.colors.primary : Qt.rgba(1, 1, 1, 0.6)
      readonly property real _cH: matugenContent.contentHeight
      readonly property real _vH: matugenContent.height
      readonly property bool _overflow: _cH > _vH && _cH > 0
      height: _overflow ? Math.min(_vH * 0.5, Math.max(16, _vH * _vH / _cH)) : 0
      y: matugenContent.y + (_overflow
        ? matugenContent.contentY / (_cH - _vH) * (_vH - height)
        : 0)
      Behavior on height { NumberAnimation { duration: 150 } }
    }

    Row {
      id: keybindsContent
      anchors.left: parent.left
      anchors.right: parent.right
      visible: settingsPanel.activeTab === "keybinds"
      spacing: 12

      Column {
        width: (parent.width - parent.spacing * 2 - 1) * 0.5
        spacing: 6

        Text {
          text: "NAVIGATION"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Repeater {
          model: [
            { key: "← / →",         action: "Navigate items" },
            { key: "↑ / ↓",         action: "Navigate rows (hex/grid)" },
            { key: "Enter",          action: "Apply wallpaper" },
            { key: "Escape",         action: "Close panel / overlay" },
            { key: "Right-click",    action: "Flip card (details)" },
            { key: "Scroll",         action: "Browse wallpapers" }
          ]
          Item {
            width: parent.width; height: 20
            Text {
              anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
              text: modelData.key
              font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.weight: Font.Bold; font.letterSpacing: 0.3
              color: settingsPanel.colors ? settingsPanel.colors.primary : Style.fallbackAccent
            }
            Text {
              anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
              text: modelData.action
              font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11)
              color: settingsPanel.colors ? settingsPanel.colors.surfaceText : Qt.rgba(1, 1, 1, 0.7)
            }
          }
        }
      }

      Rectangle {
        width: 0; height: 0; visible: false
        color: settingsPanel.colors ? Qt.rgba(settingsPanel.colors.primary.r, settingsPanel.colors.primary.g, settingsPanel.colors.primary.b, 0.1) : Qt.rgba(1, 1, 1, 0.08)
      }

      Column {
        width: (parent.width - parent.spacing * 2 - 1) * 0.5
        spacing: 6

        Text {
          text: "FILTERS & TAGS"
          font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(13); font.weight: Font.Bold; font.letterSpacing: 1.5
          color: settingsPanel.colors ? settingsPanel.colors.tertiary : Qt.rgba(1, 1, 1, 0.5)
        }

        Repeater {
          model: [
            { key: "Shift + ← / →",  action: "Cycle colour filters" },
            { key: "Shift + ↑",      action: "Toggle filter bar" },
            { key: "Shift + ↓",      action: "Toggle tag cloud" },
            { key: "Tab",            action: "Auto-complete tag" },
            { key: "Enter",          action: "Add tag (in tag input)" },
            { key: "Escape",         action: "Clear search / close" }
          ]
          Item {
            width: parent.width; height: 20
            Text {
              anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
              text: modelData.key
              font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11); font.weight: Font.Bold; font.letterSpacing: 0.3
              color: settingsPanel.colors ? settingsPanel.colors.primary : Style.fallbackAccent
            }
            Text {
              anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
              text: modelData.action
              font.family: Style.fontFamily; font.pixelSize: settingsPanel._s(11)
              color: settingsPanel.colors ? settingsPanel.colors.surfaceText : Qt.rgba(1, 1, 1, 0.7)
            }
          }
        }
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
