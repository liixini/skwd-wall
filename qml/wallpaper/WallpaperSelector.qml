import Quickshell
import Quickshell.Wayland
import QtQuick
import QtQuick.Shapes
import QtQuick.Effects
import QtQuick.Controls
import QtMultimedia
import ".."
import "../services"

Scope {
  id: wallpaperSelector

  property var colors
  property bool showing: false
  property alias selectedColorFilter: service.selectedColorFilter
  property alias selectorService: service
  property alias swService: swService
  property alias _whService: whService
  property string mainMonitor: Config.mainMonitor
  signal wallpaperChanged()
  signal uiReady()

  function _setSelectedTags(tags) {
    var hadTags = service.selectedTags.length > 0
    service.selectedTags = tags
    if (hadTags || tags.length > 0)
      service.updateFilteredModel()
  }

  function resetScroll() {
    wallpaperSelector.lastContentX = 0
    wallpaperSelector.lastIndex = 0
    sliceListView.currentIndex = 0
    if (service.filteredModel.count > 0)
      sliceListView.positionViewAtIndex(0, ListView.Center)
  }
  WallhavenService {
    id: whService
    wallpaperDir: Config.wallpaperDir
    apiKey: Config.wallhavenApiKey
  }

  SteamWorkshopService {
    id: swService
    weDir: Config.weDir
    apiKey: Config.steamApiKey
  }
  WallpaperSelectorService {
    id: service
    scriptsDir: Config.scriptsDir
    homeDir: Config.homeDir
    wallpaperDir: Config.wallpaperDir
    videoDir: Config.videoDir
    cacheBaseDir: Config.cacheDir
    weDir: Config.weDir
    weAssetsDir: Config.weAssetsDir
    showing: wallpaperSelector.showing
    onModelUpdated: {
      if (wallpaperSelector.showing && !wallpaperSelector.cardVisible) {
        wallpaperSelector.suppressWidthAnim = true
        wallpaperSelector.cardVisible = true
      }
      if (service.filteredModel.count > 0) {
        var idx = 0
        if (wallpaperSelector._restorePending) {
          wallpaperSelector._restorePending = false
          if (Config.reopenAtLastSelection)
            idx = Math.min(wallpaperSelector.lastIndex, service.filteredModel.count - 1)
        } else if (wallpaperSelector.showing && wallpaperSelector._preCommitIndex >= 0) {
          idx = Math.min(wallpaperSelector._preCommitIndex, service.filteredModel.count - 1)
        }
        wallpaperSelector._preCommitIndex = -1
        sliceListView.currentIndex = idx
        _positionTimer.posIdx = idx
        _positionTimer.restart()
      }
      if (service.filterTransitioning) {
        _snapshotFadeOut.start()
      }
    }
    onWallpaperApplied: {
      wallpaperSelector.wallpaperChanged()
      if (Config.closeOnSelection)
        wallpaperSelector.showing = false
    }
  }

  onShowingChanged: {
    if (showing) {
      if (Config.reopenAtLastSelection) {
        _restorePending = true
        DaemonClient.stateGet("ui.lastPosition", function(result) {
          if (result && result.value) {
            try {
              var pos = JSON.parse(result.value)
              wallpaperSelector.lastIndex = pos.sliceIndex ?? 0
              wallpaperSelector.lastHexCol = pos.hexCol ?? -1
              wallpaperSelector.lastHexRow = pos.hexRow ?? 0
              wallpaperSelector.lastGridIndex = pos.gridIndex ?? 0
            } catch(e) {}
          }
          _bindActiveViewModel()
          service.startCacheCheck()
          cardShowTimer.restart()
        })
      } else {
        _restorePending = true
        _bindActiveViewModel()
        service.startCacheCheck()
        cardShowTimer.restart()
      }
    } else {
      cardShowTimer.stop()
      if (Config.reopenAtLastSelection) {
        lastIndex = sliceListView.currentIndex
        lastHexCol = hexListView._selectedCol
        lastHexRow = hexListView._selectedRow
        lastGridIndex = thumbGridView.hoveredIdx
        DaemonClient.stateSet("ui.lastPosition", JSON.stringify({
          sliceIndex: lastIndex,
          hexCol: lastHexCol,
          hexRow: lastHexRow,
          gridIndex: lastGridIndex
        }))
      }
      cardVisible = false
      settingsOpen = false
      if (gridBackOverlay.overlayOpen) { gridBackOverlay.overlayOpen = false; gridBackOverlay.visible = false; gridBackOverlay.overlayItemKey = "" }
      sliceListView.cacheBuffer = 0
      sliceListView.model = null
      thumbGridView.cacheBuffer = 0
      thumbGridView.model = null
      hexListView.model = null
      gc()
    }
  }
  Connections {
    target: service
    function onRequestFilterUpdate() {
      if (service.filterTransitioning) {
        _snapshotFadeOut.stop()
        _snapshotImage.visible = false
        _snapshotImage.source = ""
      }

      wallpaperSelector._preCommitIndex = sliceListView.currentIndex

      if (service._skipCrossfade || service.filteredModel.count === 0 || !wallpaperSelector.cardVisible || wallpaperSelector.anyBrowserOpen || wallpaperSelector.isHexMode || wallpaperSelector.isGridMode) {
        service._skipCrossfade = false
        service.filterTransitioning = false
        service.commitFilteredModel()
        return
      }

      service.filterTransitioning = true
      _snapshotCommitFallback.restart()
      sliceListView.grabToImage(function(result) {
        _snapshotCommitFallback.stop()
        _snapshotImage.source = result.url
        _snapshotImage.visible = true
        _snapshotImage.opacity = 1.0
        sliceListView.cacheBuffer = 0
        service.commitFilteredModel()
      })
    }
  }

  NumberAnimation {
    id: _snapshotFadeOut
    target: _snapshotImage
    property: "opacity"
    from: 1; to: 0
    duration: Style.animNormal
    easing.type: Easing.OutCubic
    onFinished: {
      _snapshotImage.visible = false
      _snapshotImage.source = ""
      service.filterTransitioning = false
      sliceListView.cacheBuffer = wallpaperSelector.expandedWidth
    }
  }

  Timer {
    id: _snapshotCommitFallback
    interval: 150
    onTriggered: {
      if (service.filterTransitioning) {
        _snapshotImage.visible = false
        _snapshotImage.source = ""
        service.commitFilteredModel()
        service.filterTransitioning = false
        sliceListView.cacheBuffer = wallpaperSelector.expandedWidth
      }
    }
  }

  Timer {
    id: cardShowTimer
    interval: 4000
    onTriggered: wallpaperSelector.cardVisible = true
  }

  Timer {
    id: _positionTimer
    property int posIdx: 0
    interval: 0
    onTriggered: {
      console.log("[TIMER] posIdx=", posIdx, "count=", sliceListView.count, "visible=", sliceListView.visible, "contentX=", sliceListView.contentX, "width=", sliceListView.width, "height=", sliceListView.height, "contentWidth=", sliceListView.contentWidth)
      sliceListView.positionViewAtIndex(posIdx, ListView.Center)
      console.log("[TIMER] after position: contentX=", sliceListView.contentX)
      wallpaperSelector.suppressWidthAnim = false
    }
  }

  function _focusActiveList() {
    if (wallpaperSelector.tagCloudVisible) return
    if (isHexMode) hexListView.forceActiveFocus()
    else if (isGridMode) thumbGridView.forceActiveFocus()
    else sliceListView.forceActiveFocus()
  }

  Timer {
    id: focusTimer
    interval: 50
    onTriggered: wallpaperSelector._focusActiveList()
  }
  property int sliceWidth: Config.wallpaperSliceWidth
  Behavior on sliceWidth { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
  property int expandedWidth: Config.wallpaperExpandedWidth
  Behavior on expandedWidth { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
  property int sliceHeight: Config.wallpaperSliceHeight
  Behavior on sliceHeight { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
  property int skewOffset: Config.wallpaperSkewOffset
  Behavior on skewOffset { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
  property int sliceSpacing: Config.wallpaperSliceSpacing
  Behavior on sliceSpacing { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
  property bool suppressWidthAnim: false
  property int topBarHeight: 50
  property bool tagCloudVisible: false
  property bool wallhavenBrowserOpen: false
  property bool steamWorkshopBrowserOpen: false
  property bool anyBrowserOpen: wallhavenBrowserOpen || steamWorkshopBrowserOpen
  property bool isHexMode: Config.displayMode === "hex"
  property bool isGridMode: Config.displayMode === "wall"
  property bool isSliceMode: !isHexMode && !isGridMode

  onIsHexModeChanged: if (showing) _bindActiveViewModel()
  onIsGridModeChanged: if (showing) _bindActiveViewModel()

  function _bindActiveViewModel() {
    var _isSlice = !isHexMode && !isGridMode
    console.log("[BIND] _isSlice=", _isSlice, "isHexMode=", isHexMode, "isGridMode=", isGridMode, "cardVisible=", cardVisible, "showing=", showing)
    if (_isSlice) {
      sliceListView.model = Qt.binding(function() { return service.filteredModel })
      sliceListView.cacheBuffer = wallpaperSelector.expandedWidth
      _positionTimer.posIdx = Math.min(Math.max(0, sliceListView.currentIndex), Math.max(0, (service.filteredModel ? service.filteredModel.count : 1) - 1))
      console.log("[BIND] slice: count=", (service.filteredModel ? service.filteredModel.count : "null"), "currentIndex=", sliceListView.currentIndex, "posIdx=", _positionTimer.posIdx, "visible=", sliceListView.visible, "width=", sliceListView.width, "height=", sliceListView.height)
      _positionTimer.restart()
    } else {
      sliceListView.model = null
      sliceListView.cacheBuffer = 0
    }
    if (isGridMode) {
      thumbGridView.model = Qt.binding(function() { return service.filteredModel })
      thumbGridView.cacheBuffer = 300
    } else {
      thumbGridView.model = null
      thumbGridView.cacheBuffer = 0
    }
    if (isHexMode) {
      hexListView.model = Qt.binding(function() { return Math.ceil((service.filteredModel ? service.filteredModel.count : 0) / Math.max(1, hexListView._rows)) })
    } else {
      hexListView.model = null
    }
  }
  property int hexRadius: Config.hexRadius
  Behavior on hexRadius { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
  property int hexRows: Config.hexRows
  Behavior on hexRows { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
  property int hexCols: Config.hexCols
  Behavior on hexCols { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }

  property real _gridCellW: Config.gridThumbWidth + 8
  Behavior on _gridCellW { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
  property real _gridCellH: Config.gridThumbHeight + 8
  Behavior on _gridCellH { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
  property real _gridTotalW: _gridCellW * Config.gridColumns
  Behavior on _gridTotalW { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
  property int _gridTotalH: _gridCellH * Config.gridRows
  Behavior on _gridTotalH { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }

  property int cardHeight: anyBrowserOpen ? 0 : (isHexMode ? hexGridHeight : (isGridMode ? _gridTotalH + topBarHeight + 35 : sliceHeight + topBarHeight + 60))
  property int hexCardWidth: selectorPanel.width
  property int _sliceListW: Config.wallpaperExpandedWidth + (Config.wallpaperVisibleCount - 1) * (Config.wallpaperSliceWidth + Config.wallpaperSliceSpacing)
  property int cardWidth: isHexMode ? hexCardWidth : (isGridMode ? _gridTotalW + 20 : Math.max(_sliceListW + 40, 600))
  Behavior on cardWidth { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
  property int hexGridHeight: {
    var rows = hexRows
    var r = hexRadius
    var spacing = 6
    var hexH = Math.ceil(r * 1.73205)
    var stepY = hexH + spacing
    var contentH = (rows - 1) * stepY + hexH + hexH / 2
    return contentH + topBarHeight + 90
  }
  Behavior on cardHeight { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }

  property bool settingsOpen: false
  property real _settingsShift: {
    if (!settingsOpen) return 0
    var h = settingsLoader.height
    var base = h - 4
    var naturalCardY = (selectorPanel.height - cardHeight) / 2
    var settingsY = naturalCardY + base / 2 + filterBarBg.y - h - 8
    if (settingsY < 8) {
      var extra = 2 * (8 - settingsY)
      return base + extra
    }
    return base
  }
  Behavior on _settingsShift { NumberAnimation { duration: 500; easing.type: Easing.OutBack; easing.overshoot: 1.2 } }
  property real lastContentX: 0
  property int lastIndex: 0
  property int lastHexCol: -1
  property int lastHexRow: 0
  property int lastGridIndex: 0
  property bool _restorePending: false
  property int _preCommitIndex: -1
  property bool cardVisible: false
  PanelWindow {
    id: selectorPanel

    screen: Quickshell.screens.find(s => s.name === wallpaperSelector.mainMonitor)
        ?? Quickshell.screens[0]

    anchors {
      top: true
      bottom: true
      left: true
      right: true
    }
    margins {
      top: 0
      bottom: 0
      left: 0
      right: 0
    }

    visible: wallpaperSelector.showing
    color: "transparent"

    WlrLayershell.namespace: "wallpaper-selector-parallel"
    WlrLayershell.layer: WlrLayer.Overlay
    WlrLayershell.keyboardFocus: wallpaperSelector.showing ? WlrKeyboardFocus.Exclusive : WlrKeyboardFocus.None

    exclusionMode: ExclusionMode.Ignore
    Rectangle {
      anchors.fill: parent
      color: Qt.rgba(0, 0, 0, 0.5)
      opacity: wallpaperSelector.cardVisible ? 1 : 0
      Behavior on opacity { NumberAnimation { duration: Style.animMedium } }
    }
    MouseArea {
      anchors.fill: parent
      acceptedButtons: Qt.LeftButton | Qt.RightButton
      onClicked: {
        if (wallpaperSelector.anyBrowserOpen) {
          wallpaperSelector.wallhavenBrowserOpen = false
          wallpaperSelector.steamWorkshopBrowserOpen = false
        } else {
          wallpaperSelector.showing = false
        }
      }
    }
  Item {
    id: cardContainer
    width: wallpaperSelector.cardWidth
    height: wallpaperSelector.cardHeight
    anchors.centerIn: parent
    anchors.verticalCenterOffset: wallpaperSelector._settingsShift / 2
    visible: wallpaperSelector.cardVisible
    opacity: 0
    property bool animateIn: wallpaperSelector.cardVisible

    onAnimateInChanged: {
      if (animateIn) {
        opacity = 1
        focusTimer.restart()
        wallpaperSelector.uiReady()
      }
    }

    MouseArea {
      anchors.fill: parent
      onClicked: {}
    }

  Item {
    id: backgroundRect
    anchors.fill: parent

    FilterBar {
      id: filterBarBg
      anchors.horizontalCenter: parent.horizontalCenter
      anchors.top: parent.top
      anchors.topMargin: 30
      maxWidth: parent.width - 20
      z: 10
      colors: wallpaperSelector.colors
      service: service
      settingsOpen: wallpaperSelector.settingsOpen
      ollamaActive: service.ollamaActive
      cacheLoading: service.cacheLoading
      cacheProgress: service.cacheProgress
      cacheTotal: service.cacheTotal
      matugenRunning: MatugenCacheService.running
      matugenProgress: MatugenCacheService.progress
      matugenTotal: MatugenCacheService.total
      ollamaProgress: service.ollamaTaggedCount
      ollamaTotal: service.ollamaTotalThumbs
      ollamaEta: service.ollamaEta
      ollamaLogLine: service.ollamaLogLine
      videoConvertRunning: VideoConvertService.running
      videoConvertProgress: VideoConvertService.progress
      videoConvertTotal: VideoConvertService.total
      videoConvertFile: VideoConvertService.currentFile
      imageOptimizeRunning: ImageOptimizeService.running
      imageOptimizeProgress: ImageOptimizeService.progress
      imageOptimizeTotal: ImageOptimizeService.total
      imageOptimizeFile: ImageOptimizeService.currentFile
      wallhavenBrowserOpen: wallpaperSelector.wallhavenBrowserOpen
      steamWorkshopBrowserOpen: wallpaperSelector.steamWorkshopBrowserOpen
      tagCloudOpen: wallpaperSelector.tagCloudVisible
      weatherFilterActive: service.weatherFilterActive
      onSettingsToggled: { wallpaperSelector.settingsOpen = !wallpaperSelector.settingsOpen; if (!wallpaperSelector.settingsOpen) wallpaperSelector._focusActiveList() }
      onWallhavenToggled: { wallpaperSelector.settingsOpen = false; wallpaperSelector.steamWorkshopBrowserOpen = false; wallpaperSelector.wallhavenBrowserOpen = !wallpaperSelector.wallhavenBrowserOpen }
      onSteamWorkshopToggled: { wallpaperSelector.settingsOpen = false; wallpaperSelector.wallhavenBrowserOpen = false; wallpaperSelector.steamWorkshopBrowserOpen = !wallpaperSelector.steamWorkshopBrowserOpen }
      onTagCloudToggled: {
        wallpaperSelector.tagCloudVisible = !wallpaperSelector.tagCloudVisible
        if (!wallpaperSelector.tagCloudVisible)
          wallpaperSelector._setSelectedTags([])
      }
      visible: !wallpaperSelector.anyBrowserOpen
      opacity: wallpaperSelector.anyBrowserOpen ? 0 : 1
      Behavior on opacity { NumberAnimation { duration: Style.animNormal } }
    }

    }

    CacheProgressBar {
      id: cacheProgressBar
      anchors.bottom: parent.bottom
      anchors.horizontalCenter: parent.horizontalCenter
      anchors.bottomMargin: 30
      colors: wallpaperSelector.colors
      cacheLoading: service.cacheLoading
      cacheProgress: service.cacheProgress
      cacheTotal: service.cacheTotal
    }
  }

    Loader {
      id: settingsLoader
      active: wallpaperSelector.settingsOpen
      anchors.horizontalCenter: parent.horizontalCenter
      y: Math.max(8, cardContainer.y + filterBarBg.y - height - 8)
      z: 999
      sourceComponent: Component {
        SettingsPanel {
          colors: wallpaperSelector.colors
          service: wallpaperSelector.selectorService
          settingsOpen: wallpaperSelector.settingsOpen
          onCloseRequested: { wallpaperSelector.settingsOpen = false; wallpaperSelector._focusActiveList() }
        }
      }
    }
    Loader {
      id: tagCloudLoader
      active: wallpaperSelector.tagCloudVisible
      anchors.top: cardContainer.bottom
      anchors.horizontalCenter: cardContainer.horizontalCenter
      z: 5
      sourceComponent: Component {
        TagCloud {
          parentWidth: cardContainer.width
          colors: wallpaperSelector.colors
          service: wallpaperSelector.selectorService
          tagCloudVisible: true
          onEscapePressed: wallpaperSelector._focusActiveList()
          onCloseRequested: {
            wallpaperSelector.tagCloudVisible = false
            wallpaperSelector._setSelectedTags([])
            wallpaperSelector._focusActiveList()
          }
        }
      }
    }

    Loader {
      id: whBrowserLoader
      active: wallpaperSelector.wallhavenBrowserOpen
      anchors.centerIn: parent
      width: cardContainer.width - 20
      z: 6
      sourceComponent: Component {
        WallhavenBrowser {
          width: parent ? parent.width : 0
          colors: wallpaperSelector.colors
          whService: wallpaperSelector._whService
          browserVisible: true
          onEscapePressed: { wallpaperSelector.wallhavenBrowserOpen = false; wallpaperSelector._focusActiveList() }
        }
      }
    }

    Loader {
      id: swBrowserLoader
      active: wallpaperSelector.steamWorkshopBrowserOpen
      anchors.centerIn: parent
      width: cardContainer.width - 20
      z: 6
      sourceComponent: Component {
        SteamWorkshopBrowser {
          width: parent ? parent.width : 0
          colors: wallpaperSelector.colors
          swService: wallpaperSelector.swService
          browserVisible: true
          onEscapePressed: { wallpaperSelector.steamWorkshopBrowserOpen = false; wallpaperSelector._focusActiveList() }
        }
      }
    }
    ListView {
      id: sliceListView

      anchors.top: cardContainer.top
      anchors.topMargin: wallpaperSelector.topBarHeight + 15
      anchors.bottom: cardContainer.bottom
      anchors.bottomMargin: 20

      anchors.horizontalCenter: parent.horizontalCenter
      property int visibleCount: Config.wallpaperVisibleCount
      width: wallpaperSelector.expandedWidth + (visibleCount - 1) * (wallpaperSelector.sliceWidth + wallpaperSelector.sliceSpacing)
      Behavior on width { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }

      orientation: ListView.Horizontal
      model: service.filteredModel
      clip: false
      spacing: wallpaperSelector.sliceSpacing

      flickDeceleration: 1500
      maximumFlickVelocity: 3000
      boundsBehavior: Flickable.StopAtBounds
      cacheBuffer: wallpaperSelector.expandedWidth

      visible: wallpaperSelector.cardVisible && !wallpaperSelector.anyBrowserOpen && !wallpaperSelector.isHexMode && !wallpaperSelector.isGridMode

      property bool keyboardNavActive: false
      property real lastMouseX: -1
      property real lastMouseY: -1

      highlightFollowsCurrentItem: true
      highlightMoveDuration: Style.animExpand
      highlight: Item {}

      add: Transition {
        enabled: !service.filterTransitioning
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Style.animEnter; easing.type: Easing.OutCubic }
        NumberAnimation { property: "scale"; from: 0.85; to: 1; duration: Style.animEnter; easing.type: Easing.OutCubic }
      }
      remove: Transition {
        enabled: !service.filterTransitioning
        NumberAnimation { property: "opacity"; to: 0; duration: Style.animNormal; easing.type: Easing.InCubic }
      }
      displaced: Transition {
        enabled: !service.filterTransitioning
        NumberAnimation { properties: "x,y"; duration: Style.animMedium; easing.type: Easing.OutCubic }
      }
      move: Transition {
        enabled: !service.filterTransitioning
        NumberAnimation { properties: "x,y"; duration: Style.animMedium; easing.type: Easing.OutCubic }
      }

      preferredHighlightBegin: (width - wallpaperSelector.expandedWidth) / 2
      preferredHighlightEnd: (width + wallpaperSelector.expandedWidth) / 2
      highlightRangeMode: ListView.StrictlyEnforceRange

      header: Item { width: (sliceListView.width - wallpaperSelector.expandedWidth) / 2; height: 1 }
      footer: Item { width: (sliceListView.width - wallpaperSelector.expandedWidth) / 2; height: 1 }

      focus: wallpaperSelector.showing && !wallpaperSelector.tagCloudVisible
      onVisibleChanged: {
        console.log("[SLICE] onVisibleChanged visible=", visible, "count=", count, "model=", (model ? "set" : "null"), "contentX=", contentX, "width=", width, "height=", height, "contentWidth=", contentWidth)
        if (visible && !wallpaperSelector.tagCloudVisible && !wallpaperSelector.isHexMode) forceActiveFocus()
      }

      Connections {
        target: wallpaperSelector
        function onShowingChanged() {
          if (!wallpaperSelector.showing) {
            wallpaperSelector.lastContentX = sliceListView.contentX
            wallpaperSelector.lastIndex = sliceListView.currentIndex
          } else {
          if (!wallpaperSelector.tagCloudVisible)
              wallpaperSelector._focusActiveList()
          }
        }
      }
      onCountChanged: {
        console.log("[SLICE] onCountChanged count=", count, "visible=", visible, "contentX=", contentX, "contentWidth=", contentWidth)
        if (count > 0 && wallpaperSelector.showing && !wallpaperSelector._restorePending) {
          currentIndex = Math.min(currentIndex, count - 1)
        }
      }

      MouseArea {
        anchors.fill: parent
        propagateComposedEvents: true
        onWheel: function(wheel) {

          var step = 1
          if (wheel.angleDelta.y > 0 || wheel.angleDelta.x > 0) {
            sliceListView.currentIndex = Math.max(0, sliceListView.currentIndex - step)
          } else if (wheel.angleDelta.y < 0 || wheel.angleDelta.x < 0) {
            sliceListView.currentIndex = Math.min(service.filteredModel.count - 1, sliceListView.currentIndex + step)
          }
        }
        onPressed: function(mouse) { mouse.accepted = false }
        onReleased: function(mouse) { mouse.accepted = false }
        onClicked: function(mouse) { mouse.accepted = false }
      }

      Timer {
        id: wheelDebounce
        interval: 400
        onTriggered: {
          var centerX = sliceListView.contentX + sliceListView.width / 2
          var nearest = sliceListView.indexAt(centerX, sliceListView.height / 2)
          if (nearest >= 0) sliceListView.currentIndex = nearest
        }
      }

      Keys.onEscapePressed: wallpaperSelector.showing = false
      Keys.onReturnPressed: {
        if (currentIndex >= 0 && currentIndex < service.filteredModel.count) {
          const item = service.filteredModel.get(currentIndex)
          if (item.type === "we") {
            service.applyWE(item.weId)
          } else if (item.type === "video") {
            service.applyVideo(item.path)
          } else {
            service.applyStatic(item.path)
          }
        }
      }
      Keys.onPressed: function(event) {

        if (event.modifiers & Qt.ShiftModifier) {
          if (event.key === Qt.Key_Down) {
            wallpaperSelector.tagCloudVisible = !wallpaperSelector.tagCloudVisible
            if (!wallpaperSelector.tagCloudVisible)
              wallpaperSelector._setSelectedTags([])
            event.accepted = true
            return
          } else if (event.key === Qt.Key_Left) {
            if (service.selectedColorFilter === -1) {
              service.selectedColorFilter = 99
            } else if (service.selectedColorFilter === 99) {
              service.selectedColorFilter = 11
            } else if (service.selectedColorFilter === 0) {
              service.selectedColorFilter = 99
            } else {
              service.selectedColorFilter--
            }
            event.accepted = true
            return
          } else if (event.key === Qt.Key_Right) {
            if (service.selectedColorFilter === -1) {
              service.selectedColorFilter = 0
            } else if (service.selectedColorFilter === 11) {
              service.selectedColorFilter = 99
            } else if (service.selectedColorFilter === 99) {
              service.selectedColorFilter = 0
            } else {
              service.selectedColorFilter++
            }
            event.accepted = true
            return
          }
        }
        if (event.key === Qt.Key_Left && !(event.modifiers & Qt.ShiftModifier)) {
          keyboardNavActive = true
          if (currentIndex > 0) {
            currentIndex--
          }
          event.accepted = true
          return
        }

        if (event.key === Qt.Key_Right && !(event.modifiers & Qt.ShiftModifier)) {
          keyboardNavActive = true
          if (currentIndex < service.filteredModel.count - 1) {
            currentIndex++
          }
          event.accepted = true
          return
        }
      }

      delegate: SliceDelegate {
        colors: wallpaperSelector.colors
        expandedWidth: wallpaperSelector.expandedWidth
        sliceWidth: wallpaperSelector.sliceWidth
        skewOffset: wallpaperSelector.skewOffset
        service: wallpaperSelector.selectorService
        suppressWidthAnim: wallpaperSelector.suppressWidthAnim
      }
    }
    Image {
      id: _snapshotImage
      anchors.fill: sliceListView
      visible: false
      opacity: 0
      z: sliceListView.z + 1
    }

    ListView {
      id: hexListView

      anchors.top: cardContainer.top
      anchors.topMargin: wallpaperSelector.topBarHeight + 15
      anchors.bottom: cardContainer.bottom
      anchors.bottomMargin: 20
      anchors.left: cardContainer.left
      anchors.right: cardContainer.right
      visible: wallpaperSelector.cardVisible && !wallpaperSelector.anyBrowserOpen && wallpaperSelector.isHexMode

      orientation: ListView.Horizontal
      clip: true
      property int _rows: wallpaperSelector.hexRows
      property real _r: wallpaperSelector.hexRadius
      property real _gridSpacing: 6
      property real _hexW: _r * 2
      property real _hexH: Math.ceil(_r * 1.73205)
      property real _stepX: 1.5 * _r + _gridSpacing
      property real _stepY: _hexH + _gridSpacing
      property real _gridContentH: (_rows - 1) * _stepY + _hexH + _hexH / 2
      property real _yOffset: Math.max(0, (height - _gridContentH) / 2)
      property real _visibleBand: (wallpaperSelector.hexCols - 1) * _stepX + _hexW
      property real _fadeZone: (width - _visibleBand) / 2

      boundsBehavior: Flickable.StopAtBounds
      flickDeceleration: 1500
      maximumFlickVelocity: 3000
      cacheBuffer: _stepX * 2

      focus: wallpaperSelector.showing && wallpaperSelector.isHexMode && !wallpaperSelector.tagCloudVisible
      property bool _initialSnap: true
      onVisibleChanged: {
        if (visible && !wallpaperSelector.tagCloudVisible) forceActiveFocus()
        if (visible) {
          _initialSnap = true
          _restored = false
          highlightMoveDuration = 0
          var startCol
          if (Config.reopenAtLastSelection && wallpaperSelector.lastHexCol >= 0) {
            startCol = Math.min(wallpaperSelector.lastHexCol, count - 1)
            if (startCol >= 0) { currentIndex = startCol; _selectedCol = startCol; _selectedRow = wallpaperSelector.lastHexRow }
            _restored = true
          } else {
            startCol = Math.min(Math.floor(wallpaperSelector.hexCols / 2), count - 1)
            if (startCol >= 0) { currentIndex = startCol; _selectedCol = startCol; _selectedRow = 0 }
          }
          positionViewAtIndex(currentIndex, ListView.Center)
          _snapRestoreTimer.restart()
        } else {
          _restored = false
        }
      }

      Timer {
        id: _snapRestoreTimer
        interval: 50
        onTriggered: {
          hexListView.highlightMoveDuration = Style.animExpand
          hexListView._initialSnap = false
        }
      }

      model: Math.ceil((service.filteredModel ? service.filteredModel.count : 0) / Math.max(1, _rows))

      property bool _restored: false
      onCountChanged: {
        if (count > 0 && visible && !wallpaperSelector.tagCloudVisible && !_restored) {
          var startCol = Math.min(Math.floor(wallpaperSelector.hexCols / 2), count - 1)
          if (startCol >= 0) { currentIndex = startCol; _selectedCol = startCol; _selectedRow = 0 }
        }
      }

      spacing: 0

      highlightFollowsCurrentItem: true
      highlightMoveDuration: Style.animExpand
      highlight: Item {}
      preferredHighlightBegin: (width - _hexW) / 2
      preferredHighlightEnd: (width + _hexW) / 2
      highlightRangeMode: ListView.StrictlyEnforceRange

      header: Item { width: (hexListView.width - hexListView._hexW) / 2 }
      footer: Item { width: (hexListView.width - hexListView._hexW) / 2 }

      add: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Style.animEnter; easing.type: Easing.OutCubic }
        NumberAnimation { property: "scale"; from: 0.9; to: 1; duration: Style.animEnter; easing.type: Easing.OutCubic }
      }
      remove: Transition {
        NumberAnimation { property: "opacity"; to: 0; duration: Style.animNormal; easing.type: Easing.InCubic }
      }
      displaced: Transition {
        NumberAnimation { properties: "x,y"; duration: Style.animMedium; easing.type: Easing.OutCubic }
      }

      MouseArea {
        anchors.fill: parent
        propagateComposedEvents: true
        onWheel: function(wheel) {
          var step = Config.hexScrollStep
          if (wheel.angleDelta.y > 0 || wheel.angleDelta.x > 0) {
            hexListView.currentIndex = Math.max(0, hexListView.currentIndex - step)
            hexListView._selectedCol = hexListView.currentIndex
          } else if (wheel.angleDelta.y < 0 || wheel.angleDelta.x < 0) {
            hexListView.currentIndex = Math.min(hexListView.count - 1, hexListView.currentIndex + step)
            hexListView._selectedCol = hexListView.currentIndex
          }
        }
        onPressed: function(mouse) { mouse.accepted = false }
        onReleased: function(mouse) { mouse.accepted = false }
        onClicked: function(mouse) { mouse.accepted = false }
      }

      Keys.onEscapePressed: wallpaperSelector.showing = false
      Keys.onReturnPressed: {
        var flatIdx = _selectedCol * _rows + _selectedRow
        if (flatIdx >= 0 && flatIdx < service.filteredModel.count) {
          var item = service.filteredModel.get(flatIdx)
          if (item.type === "we") service.applyWE(item.weId)
          else if (item.type === "video") service.applyVideo(item.path)
          else service.applyStatic(item.path)
        }
      }

      property int _selectedCol: currentIndex
      property int _selectedRow: 0

      Keys.onPressed: function(event) {
        if (event.modifiers & Qt.ShiftModifier) {
          if (event.key === Qt.Key_Down) {
            wallpaperSelector.tagCloudVisible = !wallpaperSelector.tagCloudVisible
            if (!wallpaperSelector.tagCloudVisible)
              wallpaperSelector._setSelectedTags([])
            event.accepted = true
            return
          } else if (event.key === Qt.Key_Left) {
            if (service.selectedColorFilter === -1) service.selectedColorFilter = 99
            else if (service.selectedColorFilter === 99) service.selectedColorFilter = 11
            else if (service.selectedColorFilter === 0) service.selectedColorFilter = 99
            else service.selectedColorFilter--
            event.accepted = true
            return
          } else if (event.key === Qt.Key_Right) {
            if (service.selectedColorFilter === -1) service.selectedColorFilter = 0
            else if (service.selectedColorFilter === 11) service.selectedColorFilter = 99
            else if (service.selectedColorFilter === 99) service.selectedColorFilter = 0
            else service.selectedColorFilter++
            event.accepted = true
            return
          }
        }
        if (event.key === Qt.Key_Left && !(event.modifiers & Qt.ShiftModifier)) {
          if (currentIndex > 0) { currentIndex--; _selectedCol = currentIndex }
          event.accepted = true
          return
        }
        if (event.key === Qt.Key_Right && !(event.modifiers & Qt.ShiftModifier)) {
          if (currentIndex < count - 1) { currentIndex++; _selectedCol = currentIndex }
          event.accepted = true
          return
        }
        if (event.key === Qt.Key_Up && !(event.modifiers & Qt.ShiftModifier)) {
          if (_selectedRow > 0) _selectedRow--
          event.accepted = true
          return
        }
        if (event.key === Qt.Key_Down && !(event.modifiers & Qt.ShiftModifier)) {
          var maxRow = Math.min(_rows, service.filteredModel.count - _selectedCol * _rows) - 1
          if (_selectedRow < maxRow) _selectedRow++
          event.accepted = true
          return
        }
      }

      delegate: Item {
        id: hexCol
        width: hexListView._stepX
        height: hexListView.height
        clip: false
        property int colIdx: index

        readonly property real _colCenter: (x - hexListView.contentX) + width * 0.5
        readonly property bool _insideView: _colCenter > -hexListView._hexW && _colCenter < hexListView.width + hexListView._hexW
        readonly property bool _nearEdge: _colCenter < hexListView._fadeZone || _colCenter > (hexListView.width - hexListView._fadeZone)
        readonly property bool _nearLeft: _colCenter < hexListView.width / 2
        readonly property bool _visible: _insideView && !_nearEdge
        property real _colScale: _visible ? 1 : 0
        Behavior on _colScale { enabled: !hexListView._initialSnap; NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }

        property real _arcFactor: Config.hexArc ? Config.hexArcIntensity : 0
        Behavior on _arcFactor { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }

        readonly property real _arcOffset: {
          if (_arcFactor === 0) return 0
          var viewCenterX = hexListView.width / 2
          var normalized = (_colCenter - viewCenterX) / Math.max(1, viewCenterX)
          return -normalized * normalized * hexListView._r * _arcFactor
        }

        Repeater {
          model: Math.max(0, Math.min(hexListView._rows, service.filteredModel.count - hexCol.colIdx * hexListView._rows))

          HexDelegate {
            property int rowIdx: index
            property int flatIdx: hexCol.colIdx * hexListView._rows + rowIdx

            hexRadius: hexListView._r
            colors: wallpaperSelector.colors
            service: wallpaperSelector.selectorService
            itemData: service.filteredModel.get(flatIdx)
            isSelected: hexCol.colIdx === hexListView._selectedCol && rowIdx === hexListView._selectedRow

            x: 0
            y: hexListView._yOffset + rowIdx * hexListView._stepY + (hexCol.colIdx % 2 !== 0 ? hexListView._hexH / 2 : 0) + hexCol._arcOffset

            parallaxX: {
              var viewCenterX = hexListView.width / 2
              var normalized = (hexCol._colCenter - viewCenterX) / Math.max(1, viewCenterX)
              return -normalized * hexListView._r * 0.6
            }
            parallaxY: {
              var viewCenterY = hexListView.height / 2
              var hexCenterY = y + height / 2
              var normalized = (hexCenterY - viewCenterY) / Math.max(1, viewCenterY)
              return -normalized * hexListView._r * 0.6
            }

            scale: hexCol._colScale
            transformOrigin: hexCol._nearLeft ? Item.Left : Item.Right
            opacity: hexCol._colScale < 0.01 ? 0 : 1
            pulledOut: hexBackOverlay.overlayItemKey !== "" && hexBackOverlay.overlayItemKey === ((itemData && ((itemData.weId || "") !== "")) ? itemData.weId : (itemData ? itemData.name : ""))

            onFlipRequested: function(data, gx, gy, sourceItem) {
              hexBackOverlay.show(data, gx, gy, sourceItem)
            }
            onHoverSelected: {
              hexListView._selectedCol = hexCol.colIdx
              hexListView._selectedRow = rowIdx
            }
          }
        }
      }
    }

    GridView {
      id: thumbGridView

      anchors.top: cardContainer.top
      anchors.topMargin: wallpaperSelector.topBarHeight + 15
      anchors.bottom: cardContainer.bottom
      anchors.bottomMargin: 20
      anchors.horizontalCenter: parent.horizontalCenter
      width: wallpaperSelector._gridTotalW
      Behavior on width { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
      clip: true

      cellWidth: wallpaperSelector._gridCellW
      Behavior on cellWidth { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }
      cellHeight: wallpaperSelector._gridCellH
      Behavior on cellHeight { NumberAnimation { duration: Style.animExpand; easing.type: Easing.OutCubic } }

      model: service.filteredModel
      cacheBuffer: 300
      boundsBehavior: Flickable.StopAtBounds
      interactive: false

      property real _scrollTarget: 0
      onContentYChanged: {
        if (!_gridScrollAnim.running) _scrollTarget = contentY
      }

      NumberAnimation {
        id: _gridScrollAnim
        target: thumbGridView
        property: "contentY"
        duration: 400
        easing.type: Easing.OutCubic
      }

      function _snapScroll(delta) {
        if (!_gridScrollAnim.running) _scrollTarget = contentY
        var step = cellHeight
        _scrollTarget += (delta > 0 ? -step : step)
        var maxY = contentHeight - height
        _scrollTarget = Math.max(0, Math.min(_scrollTarget, maxY))
        _gridScrollAnim.stop()
        _gridScrollAnim.from = contentY
        _gridScrollAnim.to = _scrollTarget
        _gridScrollAnim.start()
      }

      MouseArea {
        anchors.fill: parent
        propagateComposedEvents: true
        onWheel: function(wheel) {
          thumbGridView._snapScroll(wheel.angleDelta.y)
          if (!wallpaperSelector.tagCloudVisible) thumbGridView.forceActiveFocus()
        }
        onPressed: function(mouse) { mouse.accepted = false }
        onReleased: function(mouse) { mouse.accepted = false }
        onClicked: function(mouse) { mouse.accepted = false }
      }

      visible: wallpaperSelector.cardVisible && !wallpaperSelector.anyBrowserOpen && wallpaperSelector.isGridMode

      focus: wallpaperSelector.showing && wallpaperSelector.isGridMode && !wallpaperSelector.tagCloudVisible
      onVisibleChanged: {
        if (visible && !wallpaperSelector.tagCloudVisible) forceActiveFocus()
        if (visible && Config.reopenAtLastSelection && wallpaperSelector.lastGridIndex > 0) {
          hoveredIdx = Math.min(wallpaperSelector.lastGridIndex, count - 1)
          positionViewAtIndex(hoveredIdx, GridView.Center)
        }
      }

      Keys.onEscapePressed: {
        if (gridBackOverlay.overlayOpen) gridBackOverlay.hide()
        else wallpaperSelector.showing = false
      }
      Keys.onReturnPressed: {
        if (hoveredIdx >= 0 && hoveredIdx < service.filteredModel.count) {
          var item = service.filteredModel.get(hoveredIdx)
          if (item.type === "we") service.applyWE(item.weId)
          else if (item.type === "video") service.applyVideo(item.path)
          else service.applyStatic(item.path)
        }
      }
      property int hoveredIdx: currentIndex

      function _ensureVisible(idx) {
        var row = Math.floor(idx / Config.gridColumns)
        var rowTop = row * cellHeight
        var rowBottom = rowTop + cellHeight
        if (rowTop < contentY) {
          _snapScrollTo(rowTop)
        } else if (rowBottom > contentY + height) {
          _snapScrollTo(rowBottom - height)
        }
      }

      function _snapScrollTo(target) {
        var maxY = contentHeight - height
        _scrollTarget = Math.max(0, Math.min(target, maxY))
        _gridScrollAnim.stop()
        _gridScrollAnim.from = contentY
        _gridScrollAnim.to = _scrollTarget
        _gridScrollAnim.start()
      }

      Keys.onUpPressed: function(event) {
        if (event.modifiers & Qt.ShiftModifier) { event.accepted = false; return }
        var newIdx = currentIndex - Config.gridColumns
        if (newIdx >= 0) {
          currentIndex = newIdx
          hoveredIdx = newIdx
          _ensureVisible(newIdx)
        }
      }
      Keys.onDownPressed: function(event) {
        if (event.modifiers & Qt.ShiftModifier) {
          wallpaperSelector.tagCloudVisible = !wallpaperSelector.tagCloudVisible
          if (!wallpaperSelector.tagCloudVisible)
            wallpaperSelector._setSelectedTags([])
          event.accepted = true
          return
        }
        var newIdx = currentIndex + Config.gridColumns
        if (newIdx < count) {
          currentIndex = newIdx
          hoveredIdx = newIdx
          _ensureVisible(newIdx)
        }
      }
      Keys.onLeftPressed: function(event) {
        if (event.modifiers & Qt.ShiftModifier) {
          if (service.selectedColorFilter === -1) service.selectedColorFilter = 99
          else if (service.selectedColorFilter === 99) service.selectedColorFilter = 11
          else if (service.selectedColorFilter === 0) service.selectedColorFilter = 99
          else service.selectedColorFilter--
          event.accepted = true
          return
        }
        if (currentIndex > 0) {
          currentIndex--
          hoveredIdx = currentIndex
          _ensureVisible(currentIndex)
        }
      }
      Keys.onRightPressed: function(event) {
        if (event.modifiers & Qt.ShiftModifier) {
          if (service.selectedColorFilter === -1) service.selectedColorFilter = 0
          else if (service.selectedColorFilter === 11) service.selectedColorFilter = 99
          else if (service.selectedColorFilter === 99) service.selectedColorFilter = 0
          else service.selectedColorFilter++
          event.accepted = true
          return
        }
        if (currentIndex < count - 1) {
          currentIndex++
          hoveredIdx = currentIndex
          _ensureVisible(currentIndex)
        }
      }

      highlightMoveDuration: Style.animNormal
      highlight: Item {}

      ScrollBar.vertical: ScrollBar {
        policy: ScrollBar.AsNeeded
        width: 4
        contentItem: Rectangle {
          radius: 2
          color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.primary.r, wallpaperSelector.colors.primary.g, wallpaperSelector.colors.primary.b, 0.4)
                                          : Qt.rgba(1, 1, 1, 0.3)
        }
      }

      add: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Style.animEnter; easing.type: Easing.OutCubic }
        NumberAnimation { property: "scale"; from: 0.85; to: 1; duration: Style.animEnter; easing.type: Easing.OutBack; easing.overshoot: 1.2 }
      }
      remove: Transition {
        NumberAnimation { property: "opacity"; to: 0; duration: Style.animVeryFast; easing.type: Easing.InCubic }
      }
      displaced: Transition {
        NumberAnimation { properties: "x,y"; duration: Style.animFast; easing.type: Easing.OutCubic }
      }

      delegate: Item {
        id: gridThumbDelegate
        width: thumbGridView.cellWidth
        height: thumbGridView.cellHeight

        required property int index
        required property var model

        property string videoPath: model.videoFile ? model.videoFile : ""
        property bool hasVideo: videoPath.length > 0 && Config.videoPreviewEnabled
        property bool videoActive: false

        onVisibleChanged: {
            if (!visible) { _gridVideoDelay.stop(); videoActive = false }
        }

        Connections {
            target: thumbGridView
            function onHoveredIdxChanged() {
                if (thumbGridView.hoveredIdx === gridThumbDelegate.index && gridThumbDelegate.hasVideo) {
                    _gridVideoDelay.restart()
                } else {
                    _gridVideoDelay.stop()
                    gridThumbDelegate.videoActive = false
                }
            }
        }

        Timer {
            id: _gridVideoDelay
            interval: 600
            onTriggered: gridThumbDelegate.videoActive = true
        }

        property real _entryOpacity: 0.8

        Behavior on _entryOpacity { NumberAnimation { duration: 300; easing.type: Easing.OutQuad } }

        opacity: _entryOpacity

        readonly property real entryViewY: y - thumbGridView.contentY
        readonly property bool entryInView: entryViewY + height > 0 && entryViewY < thumbGridView.height

        onEntryInViewChanged: {
          if (entryInView) _entryOpacity = 1.0
          else _entryOpacity = 0.8
        }

        Component.onCompleted: {
          if (entryInView) _entryOpacity = 1.0
        }

        Rectangle {
          id: gridCardRect
          anchors.fill: parent; anchors.margins: 4; radius: 6
          color: "transparent"

          border.width: thumbGridView.hoveredIdx === gridThumbDelegate.index ? 2 : 0
          border.color: wallpaperSelector.colors ? wallpaperSelector.colors.primary : "#ff8800"
          Behavior on border.width { NumberAnimation { duration: Style.animFast; easing.type: Easing.OutQuad } }

          property bool _pulledOut: gridBackOverlay.overlayItemKey !== "" && gridBackOverlay.overlayItemKey === ((gridThumbDelegate.model.weId || "") !== "" ? gridThumbDelegate.model.weId : gridThumbDelegate.model.name)
          visible: !_pulledOut

          Rectangle {
            anchors.fill: parent; anchors.margins: gridCardRect.border.width; radius: 5
            color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.surface.r, wallpaperSelector.colors.surface.g, wallpaperSelector.colors.surface.b, 0.6) : Qt.rgba(0.12, 0.14, 0.18, 0.6)
            clip: true

          Image {
            id: gridThumbImg
            anchors.fill: parent
            source: gridThumbDelegate.model.thumb ? ImageService.fileUrl(gridThumbDelegate.model.thumb) : ""
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            smooth: true
            cache: false
            sourceSize.width: Config.gridThumbWidth
            sourceSize.height: Config.gridThumbHeight
            opacity: status === Image.Ready ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: Style.animNormal; easing.type: Easing.OutCubic } }
          }

          Loader {
              id: _gridVideoLoader
              anchors.fill: parent
              active: gridThumbDelegate.videoActive
              visible: false
              layer.enabled: active

              sourceComponent: Video {
                  anchors.fill: parent
                  source: ImageService.fileUrl(gridThumbDelegate.videoPath)
                  fillMode: VideoOutput.PreserveAspectCrop
                  loops: MediaPlayer.Infinite
                  muted: true
                  Component.onCompleted: play()
              }
          }

          Item {
              anchors.fill: parent
              visible: _gridVideoLoader.active && _gridVideoLoader.status === Loader.Ready

              ShaderEffectSource {
                  anchors.fill: parent
                  sourceItem: _gridVideoLoader
                  live: true
              }
          }

          Rectangle {
            id: gridSkeleton
            anchors.fill: parent; radius: 6
            visible: opacity > 0
            opacity: gridThumbImg.status === Image.Ready ? 0 : 1
            Behavior on opacity { NumberAnimation { duration: Style.animNormal; easing.type: Easing.OutCubic } }
            color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.surfaceVariant.r, wallpaperSelector.colors.surfaceVariant.g, wallpaperSelector.colors.surfaceVariant.b, 0.8) : Qt.rgba(0.18, 0.20, 0.25, 0.8)

            Rectangle {
              id: gridShimmer
              width: parent.width * 0.5; height: parent.height; radius: 6
              opacity: 0.35
              gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0.0; color: "transparent" }
                GradientStop { position: 0.5; color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.surfaceText.r, wallpaperSelector.colors.surfaceText.g, wallpaperSelector.colors.surfaceText.b, 0.08) : Qt.rgba(1, 1, 1, 0.08) }
                GradientStop { position: 1.0; color: "transparent" }
              }
              NumberAnimation on x {
                from: -gridShimmer.width; to: gridSkeleton.width
                duration: 1200; loops: Animation.Infinite
                running: gridSkeleton.visible
              }
            }

            Text {
              anchors.centerIn: parent
              text: "\u{f0553}"
              font.family: Style.fontFamilyNerdIcons; font.pixelSize: 22
              color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.surfaceText.r, wallpaperSelector.colors.surfaceText.g, wallpaperSelector.colors.surfaceText.b, 0.15) : Qt.rgba(1,1,1,0.1)
            }
          }

          MouseArea {
            id: gridThumbMouse
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            hoverEnabled: true
            acceptedButtons: Qt.LeftButton | Qt.RightButton
            onContainsMouseChanged: {
              if (containsMouse) {
                thumbGridView.hoveredIdx = gridThumbDelegate.index
                if (!wallpaperSelector.tagCloudVisible) thumbGridView.forceActiveFocus()
              }
            }
            onClicked: function(mouse) {
              if (!wallpaperSelector.tagCloudVisible) thumbGridView.forceActiveFocus()
              if (mouse.button === Qt.RightButton) {
                var gpos = gridThumbDelegate.mapToItem(null, gridThumbDelegate.width / 2, gridThumbDelegate.height / 2)
                var d = gridThumbDelegate.model
                gridBackOverlay.show({
                  name: d.name, path: d.path, thumb: d.thumb, type: d.type,
                  weId: d.weId || "", favourite: d.favourite, videoFile: d.videoFile || ""
                }, gpos.x, gpos.y, gridThumbDelegate)
              } else {
                var d = gridThumbDelegate.model
                if (d.type === "we") service.applyWE(d.weId)
                else if (d.type === "video") service.applyVideo(d.path)
                else service.applyStatic(d.path)
              }
            }
          }

          Rectangle {
            anchors.bottom: parent.bottom; anchors.left: parent.left
            anchors.margins: 4
            width: gridTypeBadge.implicitWidth + 6; height: 14; radius: 3
            color: Qt.rgba(0, 0, 0, 0.6)
            Text {
              id: gridTypeBadge
              anchors.centerIn: parent
              text: (gridThumbDelegate.model.type === "video" || gridThumbDelegate.model.videoFile) ? "VID" : (gridThumbDelegate.model.type === "static" ? "PIC" : "WE")
              font.family: Style.fontFamily; font.pixelSize: 8; font.weight: Font.Bold
              color: wallpaperSelector.colors ? wallpaperSelector.colors.primary : "#ff8800"
            }
          }

          Rectangle {
            anchors.top: parent.top; anchors.left: parent.left
            anchors.margins: 4
            width: 18; height: 18; radius: 9
            color: gridThumbDelegate.videoActive ? (wallpaperSelector.colors ? wallpaperSelector.colors.primary : Style.fallbackAccent) : Qt.rgba(0, 0, 0, 0.7)
            border.width: 1
            border.color: gridThumbDelegate.videoActive
                ? "transparent"
                : (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.primary.r, wallpaperSelector.colors.primary.g, wallpaperSelector.colors.primary.b, 0.6) : Qt.rgba(1,1,1,0.4))
            visible: gridThumbDelegate.hasVideo
            z: 5

            Behavior on color { ColorAnimation { duration: Style.animFast } }

            Text {
              anchors.centerIn: parent; anchors.horizontalCenterOffset: 1
              text: "\u25b6"; font.pixelSize: 7
              color: gridThumbDelegate.videoActive
                  ? (wallpaperSelector.colors ? wallpaperSelector.colors.primaryText : "#000")
                  : (wallpaperSelector.colors ? wallpaperSelector.colors.primary : Style.fallbackAccent)
            }
          }

          Text {
            anchors.top: parent.top; anchors.right: parent.right
            anchors.margins: 4
            text: "\u{f0134}"
            font.family: Style.fontFamilyNerdIcons; font.pixelSize: 14
            color: wallpaperSelector.colors ? wallpaperSelector.colors.primary : "#ff8800"
            visible: gridThumbDelegate.model.favourite === true
          }
          }
        }
      }
    }

    Item {
      id: gridBackOverlay
      anchors.fill: parent
      visible: false
      z: 200

      property var overlayData: null
      property string overlayItemKey: ""
      property var _sourceItem: null
      property real sourceX: 0
      property real sourceY: 0
      property real _openContentY: 0
      property bool overlayOpen: false
      property var _gridMeta: null

      readonly property real bigW: Math.min(Config.gridThumbWidth * 2.5, 600)
      readonly property real bigH: Math.min(Config.gridThumbHeight * 2.5, 500)

      onOverlayOpenChanged: {
        if (overlayOpen && overlayData && overlayData.type !== "we") {
          var key = ImageService.thumbKey(overlayData.thumb, overlayData.name)
          _gridMeta = FileMetadataService.getMetadata(key)
          if (!_gridMeta)
            FileMetadataService.probeIfNeeded(key, overlayData.path, overlayData.type === "video" ? "video" : "image")
        }
      }
      Connections {
        target: FileMetadataService
        enabled: gridBackOverlay.overlayOpen
        function onMetadataReady(key) {
          if (!gridBackOverlay.overlayData) return
          var myKey = ImageService.thumbKey(gridBackOverlay.overlayData.thumb, gridBackOverlay.overlayData.name)
          if (key === myKey)
            gridBackOverlay._gridMeta = FileMetadataService.getMetadata(key)
        }
      }

      function show(data, gx, gy, sourceItem) {
        gridTagField._syncing = true; gridTagField.text = ""; gridTagField._sessionTags = []; gridTagField._syncing = false
        overlayData = data
        overlayItemKey = (data.weId || "") !== "" ? data.weId : data.name
        _sourceItem = sourceItem || null
        _openContentY = thumbGridView.contentY
        var local = gridBackOverlay.mapFromItem(null, gx, gy)
        sourceX = local.x
        sourceY = local.y
        visible = true
        overlayOpen = true
      }

      function hide() {
        var scrollDelta = thumbGridView.contentY - _openContentY
        sourceY -= scrollDelta
        _openContentY = thumbGridView.contentY
        overlayOpen = false
      }

      Rectangle {
        anchors.fill: parent
        color: Qt.rgba(0, 0, 0, gridBackOverlay.overlayOpen ? 0.55 : 0)
        Behavior on color { ColorAnimation { duration: Style.animNormal } }
        MouseArea {
          anchors.fill: parent
          acceptedButtons: Qt.LeftButton | Qt.RightButton
          onClicked: gridBackOverlay.hide()
        }
      }

      states: [
        State {
          name: "hidden"
          when: !gridBackOverlay.overlayOpen
          PropertyChanges {
            target: gridCard
            x: gridBackOverlay.sourceX - gridCard.width / 2
            y: gridBackOverlay.sourceY - gridCard.height / 2
            scale: Config.gridThumbWidth / gridBackOverlay.bigW
            opacity: 0
          }
          PropertyChanges { target: gridCardRotation; angle: 0 }
        },
        State {
          name: "visible"
          when: gridBackOverlay.overlayOpen
          PropertyChanges {
            target: gridCard
            x: (gridBackOverlay.width - gridCard.width) / 2
            y: (gridBackOverlay.height - gridCard.height) / 2
            scale: 1
            opacity: 1
          }
          PropertyChanges { target: gridCardRotation; angle: 180 }
        }
      ]

      transitions: [
        Transition {
          from: "hidden"; to: "visible"
          SequentialAnimation {
            PropertyAction { target: gridBackOverlay; property: "visible"; value: true }
            ParallelAnimation {
              NumberAnimation { target: gridCard; properties: "x,y,scale,opacity"; duration: Style.animSlow; easing.type: Easing.OutCubic }
              NumberAnimation { target: gridCardRotation; property: "angle"; duration: Style.animSlow; easing.type: Easing.InOutQuad }
            }
          }
        },
        Transition {
          from: "visible"; to: "hidden"
          SequentialAnimation {
            ParallelAnimation {
              NumberAnimation { target: gridCard; properties: "x,y,scale"; duration: Style.animSlow; easing.type: Easing.InOutCubic }
              NumberAnimation { target: gridCardRotation; property: "angle"; duration: Style.animSlow; easing.type: Easing.InOutQuad }
              SequentialAnimation {
                PauseAnimation { duration: Style.animSlow * 0.7 }
                NumberAnimation { target: gridCard; property: "opacity"; duration: Style.animSlow * 0.3; easing.type: Easing.InQuad }
              }
            }
            PropertyAction { target: gridBackOverlay; property: "visible"; value: false }
            PropertyAction { target: gridBackOverlay; property: "overlayItemKey"; value: "" }
            PropertyAction { target: gridBackOverlay; property: "_sourceItem"; value: null }
          }
        }
      ]

      Item {
        id: gridCard
        width: gridBackOverlay.bigW
        height: gridBackOverlay.bigH
        transformOrigin: Item.Center

        transform: Rotation {
          id: gridCardRotation
          origin.x: gridCard.width / 2
          origin.y: gridCard.height / 2
          axis { x: 0; y: 1; z: 0 }
          angle: 0
        }

        Item {
          id: gridFrontFace
          anchors.fill: parent
          visible: gridCardRotation.angle < 90

          Rectangle {
            anchors.fill: parent; radius: 12
            color: wallpaperSelector.colors ? wallpaperSelector.colors.surfaceContainer : "#1a1a2e"
            clip: true

            Image {
              anchors.fill: parent
              source: gridBackOverlay.overlayData && gridBackOverlay.overlayData.thumb
                ? ImageService.fileUrl(gridBackOverlay.overlayData.thumb) : ""
              fillMode: Image.PreserveAspectCrop
              smooth: true; asynchronous: true; cache: false
              sourceSize.width: gridBackOverlay.bigW
              sourceSize.height: gridBackOverlay.bigH
            }
          }

          Rectangle {
            anchors.fill: parent; radius: 12
            color: "transparent"
            border.width: 2
            border.color: wallpaperSelector.colors ? wallpaperSelector.colors.primary : Style.fallbackAccent
          }
        }

        Item {
          id: gridBackFace
          anchors.fill: parent
          visible: gridCardRotation.angle >= 90
          transform: Rotation {
            origin.x: gridBackFace.width / 2; origin.y: gridBackFace.height / 2
            axis { x: 0; y: 1; z: 0 }
            angle: 180
          }

          Rectangle {
            anchors.fill: parent; radius: 12
            color: wallpaperSelector.colors ? wallpaperSelector.colors.surfaceContainer : "#1a1a2e"
            clip: true

            MouseArea {
              anchors.fill: parent
              acceptedButtons: Qt.RightButton
              z: -1
              onClicked: gridBackOverlay.hide()
            }

            Image {
              anchors.fill: parent
              source: gridBackOverlay.overlayData && gridBackOverlay.overlayData.thumb
                ? ImageService.fileUrl(gridBackOverlay.overlayData.thumb) : ""
              fillMode: Image.PreserveAspectCrop; opacity: 0.08
              sourceSize.width: 120
              sourceSize.height: 68
              asynchronous: true; cache: false
            }

            Column {
              id: gridBackContent
              anchors.centerIn: parent
              width: parent.width * 0.8
              spacing: 6

              Text {
                width: parent.width
                text: gridBackOverlay.overlayData ? gridBackOverlay.overlayData.name.replace(/\.[^/.]+$/, "").toUpperCase() : ""
                color: wallpaperSelector.colors ? wallpaperSelector.colors.tertiary : "#8bceff"
                font.family: Style.fontFamily; font.pixelSize: 15; font.weight: Font.Bold; font.letterSpacing: 1.2
                horizontalAlignment: Text.AlignHCenter; wrapMode: Text.Wrap; elide: Text.ElideRight; maximumLineCount: 2
              }

              Row {
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: 0
                visible: gridBackOverlay.overlayData && gridBackOverlay.overlayData.type !== "we"
                Text {
                  text: gridBackOverlay.overlayData ? FileMetadataService.formatExt(gridBackOverlay.overlayData.name) : ""
                  color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.tertiary.r, wallpaperSelector.colors.tertiary.g, wallpaperSelector.colors.tertiary.b, 0.6) : Qt.rgba(1,1,1,0.35)
                  font.family: Style.fontFamily; font.pixelSize: 11; font.weight: Font.Medium; font.letterSpacing: 0.8
                }
                Text {
                  text: "  \u2022  "; color: Qt.rgba(1,1,1,0.15); font.family: Style.fontFamily; font.pixelSize: 11
                }
                Text {
                  text: gridBackOverlay._gridMeta ? (gridBackOverlay._gridMeta.width + " \u00d7 " + gridBackOverlay._gridMeta.height) : "\u2013"
                  color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.tertiary.r, wallpaperSelector.colors.tertiary.g, wallpaperSelector.colors.tertiary.b, 0.6) : Qt.rgba(1,1,1,0.35)
                  font.family: Style.fontFamily; font.pixelSize: 11; font.weight: Font.Medium; font.letterSpacing: 0.5
                }
                Text {
                  text: "  \u2022  "; color: Qt.rgba(1,1,1,0.15); font.family: Style.fontFamily; font.pixelSize: 11
                }
                Text {
                  text: gridBackOverlay._gridMeta ? FileMetadataService.formatSize(gridBackOverlay._gridMeta.filesize) : "\u2013"
                  color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.tertiary.r, wallpaperSelector.colors.tertiary.g, wallpaperSelector.colors.tertiary.b, 0.6) : Qt.rgba(1,1,1,0.35)
                  font.family: Style.fontFamily; font.pixelSize: 11; font.weight: Font.Medium; font.letterSpacing: 0.5
                }
              }

              Rectangle { width: parent.width; height: 1; color: Qt.rgba(1, 1, 1, 0.08) }

              Item {
                width: parent.width; height: 26
                Text {
                  anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
                  text: "FAVOURITE"
                  color: wallpaperSelector.colors ? wallpaperSelector.colors.tertiary : "#8bceff"
                  font.family: Style.fontFamily; font.pixelSize: 12; font.weight: Font.Medium; font.letterSpacing: 0.5
                }
                Item {
                  id: gridFavToggle
                  anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                  width: 44; height: 22
                  property bool checked: false
                  Connections {
                    target: gridBackOverlay
                    function onOverlayOpenChanged() {
                      if (gridBackOverlay.overlayOpen && gridBackOverlay.overlayData) {
                        var key = (gridBackOverlay.overlayData.weId || "") !== "" ? gridBackOverlay.overlayData.weId : gridBackOverlay.overlayData.name
                        gridFavToggle.checked = wallpaperSelector.selectorService ? !!wallpaperSelector.selectorService.favouritesDb[key] : false
                      }
                    }
                  }
                  Canvas {
                    anchors.fill: parent
                    property bool isOn: gridFavToggle.checked
                    property color fillColor: isOn
                      ? (wallpaperSelector.colors ? wallpaperSelector.colors.primary : Style.fallbackAccent)
                      : Qt.rgba(1, 1, 1, 0.15)
                    onFillColorChanged: requestPaint(); onIsOnChanged: requestPaint()
                    onPaint: {
                      var ctx = getContext("2d"); ctx.clearRect(0, 0, width, height)
                      var sk = 6; ctx.fillStyle = fillColor; ctx.beginPath()
                      ctx.moveTo(sk, 0); ctx.lineTo(width, 0); ctx.lineTo(width - sk, height); ctx.lineTo(0, height)
                      ctx.closePath(); ctx.fill()
                    }
                  }
                  Canvas {
                    width: 20; height: 16; y: 3
                    x: gridFavToggle.checked ? parent.width - width - 3 : 3
                    Behavior on x { NumberAnimation { duration: Style.animFast; easing.type: Easing.OutCubic } }
                    property color knobColor: gridFavToggle.checked
                      ? (wallpaperSelector.colors ? wallpaperSelector.colors.primaryText : "#000")
                      : (wallpaperSelector.colors ? wallpaperSelector.colors.surfaceText : "#fff")
                    onKnobColorChanged: requestPaint()
                    onPaint: {
                      var ctx = getContext("2d"); ctx.clearRect(0, 0, width, height)
                      var sk = 4; ctx.fillStyle = knobColor; ctx.beginPath()
                      ctx.moveTo(sk, 0); ctx.lineTo(width, 0); ctx.lineTo(width - sk, height); ctx.lineTo(0, height)
                      ctx.closePath(); ctx.fill()
                    }
                  }
                  MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: {
                      if (!gridBackOverlay.overlayData) return
                      gridFavToggle.checked = !gridFavToggle.checked
                      wallpaperSelector.selectorService.toggleFavourite(gridBackOverlay.overlayData.name, gridBackOverlay.overlayData.weId || "")
                    }
                  }
                }
              }

              Rectangle { width: parent.width; height: 1; color: Qt.rgba(1, 1, 1, 0.08) }

              Item {
                width: parent.width; height: 24
                Rectangle {
                  anchors.fill: parent
                  color: gridTagField.activeFocus
                    ? (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.surface.r, wallpaperSelector.colors.surface.g, wallpaperSelector.colors.surface.b, 0.5) : Qt.rgba(0, 0, 0, 0.3))
                    : "transparent"
                  border.width: 1
                  border.color: gridTagField.activeFocus
                    ? (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.primary.r, wallpaperSelector.colors.primary.g, wallpaperSelector.colors.primary.b, 0.5) : Qt.rgba(1, 1, 1, 0.3))
                    : (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.outline.r, wallpaperSelector.colors.outline.g, wallpaperSelector.colors.outline.b, 0.2) : Qt.rgba(1, 1, 1, 0.1))
                  Behavior on color { ColorAnimation { duration: Style.animVeryFast } }
                  Behavior on border.color { ColorAnimation { duration: Style.animVeryFast } }
                }
                TextInput {
                  id: gridTagField
                  anchors.fill: parent; anchors.leftMargin: 10; anchors.rightMargin: 10
                  verticalAlignment: TextInput.AlignVCenter
                  font.family: Style.fontFamily; font.pixelSize: 11; font.letterSpacing: 0.3
                  color: wallpaperSelector.colors ? wallpaperSelector.colors.surfaceText : "#fff"
                  clip: true
                  property var _sessionTags: []
                  property bool _syncing: false
                  onTextChanged: {
                    if (_syncing) return
                    if (!gridBackOverlay.overlayData) return
                    var raw = text.toLowerCase()
                    var words = raw.split(/\s+/).filter(function(w) { return w.length > 0 })
                    var wpTags = wallpaperSelector.selectorService.getWallpaperTags(gridTagsSection.wpName, gridTagsSection.wpWeId).slice()
                    var changed = false
                    for (var i = 0; i < words.length; i++) {
                      if (_sessionTags.indexOf(words[i]) === -1) _sessionTags.push(words[i])
                      if (wpTags.indexOf(words[i]) === -1) { wpTags.push(words[i]); changed = true }
                    }
                    var toRemove = []
                    for (var k = 0; k < _sessionTags.length; k++) {
                      if (words.indexOf(_sessionTags[k]) === -1) toRemove.push(_sessionTags[k])
                    }
                    for (var r = 0; r < toRemove.length; r++) {
                      var si = _sessionTags.indexOf(toRemove[r])
                      if (si !== -1) _sessionTags.splice(si, 1)
                      var wi = wpTags.indexOf(toRemove[r])
                      if (wi !== -1) { wpTags.splice(wi, 1); changed = true }
                    }
                    if (changed) wallpaperSelector.selectorService.setWallpaperTags(gridTagsSection.wpName, gridTagsSection.wpWeId, wpTags)
                  }
                  Keys.onReturnPressed: function(event) { event.accepted = true }
                  Keys.onEscapePressed: { _syncing = true; text = ""; _sessionTags = []; _syncing = false; gridBackOverlay.hide() }
                  Text {
                    anchors.fill: parent; verticalAlignment: Text.AlignVCenter
                    text: "+ ADD TAG"; font.family: Style.fontFamily; font.pixelSize: 11; font.letterSpacing: 1
                    color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.surfaceText.r, wallpaperSelector.colors.surfaceText.g, wallpaperSelector.colors.surfaceText.b, 0.25) : Qt.rgba(1, 1, 1, 0.2)
                    visible: !parent.text && !parent.activeFocus
                  }
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.IBeamCursor; z: -1; onClicked: gridTagField.forceActiveFocus() }
              }

              Item {
                id: gridTagsSection
                width: parent.width
                height: Math.min(Math.max(30, gridTagsFlow.implicitHeight + 10), gridBackOverlay.bigH * 0.3)
                clip: true

                property string wpName: gridBackOverlay.overlayData ? gridBackOverlay.overlayData.name : ""
                property string wpWeId: gridBackOverlay.overlayData ? (gridBackOverlay.overlayData.weId || "") : ""
                property var currentTags: {
                  if (!gridBackOverlay.overlayOpen) return []
                  var db = wallpaperSelector.selectorService ? wallpaperSelector.selectorService.tagsDb : null
                  if (!db) return []
                  var key = gridTagsSection.wpWeId ? gridTagsSection.wpWeId : ImageService.thumbKey(gridBackOverlay.overlayData ? gridBackOverlay.overlayData.thumb : "", gridTagsSection.wpName)
                  return db[key] || []
                }

                Flickable {
                  anchors.fill: parent; contentHeight: gridTagsFlow.implicitHeight
                  clip: true; flickableDirection: Flickable.VerticalFlick; boundsBehavior: Flickable.StopAtBounds
                  Flow {
                    id: gridTagsFlow; width: parent.width; spacing: 5
                    Repeater {
                      model: gridTagsSection.currentTags
                      Rectangle {
                        property bool hovered: _gridTagMa.containsMouse
                        width: _gridTagTxt.implicitWidth + 30; height: 28; radius: 4
                        color: hovered ? (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.surfaceVariant.r, wallpaperSelector.colors.surfaceVariant.g, wallpaperSelector.colors.surfaceVariant.b, 0.5) : Qt.rgba(1,1,1,0.15)) : "transparent"
                        border.width: 1
                        border.color: hovered
                          ? (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.primary.r, wallpaperSelector.colors.primary.g, wallpaperSelector.colors.primary.b, 0.7) : Qt.rgba(1,1,1,0.3))
                          : (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.outline.r, wallpaperSelector.colors.outline.g, wallpaperSelector.colors.outline.b, 0.5) : Qt.rgba(1,1,1,0.15))
                        Behavior on color { ColorAnimation { duration: Style.animVeryFast } }
                        Behavior on border.color { ColorAnimation { duration: Style.animVeryFast } }
                        transform: Matrix4x4 { matrix: Qt.matrix4x4(1, -0.08, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1) }
                        Text {
                          id: _gridTagTxt; anchors.left: parent.left; anchors.leftMargin: 8; anchors.verticalCenter: parent.verticalCenter
                          text: modelData.toUpperCase(); color: wallpaperSelector.colors ? wallpaperSelector.colors.tertiary : "#8bceff"
                          font.family: Style.fontFamily; font.pixelSize: 12; font.weight: Font.Medium; font.letterSpacing: 0.5
                        }
                        Text {
                          anchors.right: parent.right; anchors.rightMargin: 6; anchors.verticalCenter: parent.verticalCenter
                          text: "\u{f0156}"; font.family: Style.fontFamilyNerdIcons; font.pixelSize: 11
                          color: parent.hovered ? (wallpaperSelector.colors ? wallpaperSelector.colors.primary : "#ff6b6b") : Qt.rgba(1,1,1,0.25)
                          Behavior on color { ColorAnimation { duration: Style.animVeryFast } }
                        }
                        MouseArea {
                          id: _gridTagMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                          onClicked: {
                            var tags = wallpaperSelector.selectorService.getWallpaperTags(gridTagsSection.wpName, gridTagsSection.wpWeId).slice()
                            var idx = tags.indexOf(modelData); if (idx !== -1) tags.splice(idx, 1)
                            wallpaperSelector.selectorService.setWallpaperTags(gridTagsSection.wpName, gridTagsSection.wpWeId, tags)
                          }
                        }
                      }
                    }
                  }
                }
                Text {
                  anchors.centerIn: parent; visible: gridTagsSection.currentTags.length === 0
                  text: "NO TAGS"; color: Qt.rgba(1,1,1,0.15); font.family: Style.fontFamily; font.pixelSize: 12; font.letterSpacing: 2
                }
              }

              Row {
                id: gridActionRow
                width: parent.width; height: 32; spacing: 8

                ActionButton {
                  width: gridBackOverlay.overlayData && gridBackOverlay.overlayData.type === "we" ? (parent.width - parent.spacing * 2) / 3 : (parent.width - parent.spacing) / 2
                  colors: wallpaperSelector.colors
                  icon: "\u{f0208}"; label: "VIEW"
                  onClicked: { if (!gridBackOverlay.overlayData) return; var p = gridBackOverlay.overlayData.path; Qt.openUrlExternally(ImageService.fileUrl(p.substring(0, p.lastIndexOf("/")))); gridBackOverlay.hide() }
                }

                ActionButton {
                  width: gridBackOverlay.overlayData && gridBackOverlay.overlayData.type === "we" ? (parent.width - parent.spacing * 2) / 3 : (parent.width - parent.spacing) / 2
                  colors: wallpaperSelector.colors
                  icon: "\u{f0a79}"; label: "DELETE"; danger: true
                  onClicked: { if (!gridBackOverlay.overlayData) return; wallpaperSelector.selectorService.deleteWallpaperItem(gridBackOverlay.overlayData.type, gridBackOverlay.overlayData.name, gridBackOverlay.overlayData.weId || ""); gridBackOverlay.hide() }
                }

                ActionButton {
                  visible: gridBackOverlay.overlayData && gridBackOverlay.overlayData.type === "we"
                  width: visible ? (parent.width - parent.spacing * 2) / 3 : 0
                  colors: wallpaperSelector.colors
                  icon: "\u{f0bef}"; label: "STEAM"
                  onClicked: { wallpaperSelector.selectorService.openSteamPage(gridBackOverlay.overlayData.weId || ""); gridBackOverlay.hide() }
                }
              }
            }
          }

          Rectangle {
            anchors.fill: parent; radius: 12
            color: "transparent"
            border.width: 2.5
            border.color: wallpaperSelector.colors ? wallpaperSelector.colors.primary : Style.fallbackAccent
          }
        }

      }
    }

    Item {
      id: hexBackOverlay
      anchors.fill: parent
      visible: false
      z: 200

      property var overlayData: null
      property string overlayItemKey: ""
      property var _sourceItem: null
      property real sourceX: 0
      property real sourceY: 0
      property real _openContentX: 0
      property bool overlayOpen: false
      property var _hexMeta: null

      readonly property real bigR: wallpaperSelector.hexRadius * 3

      onOverlayOpenChanged: {
        if (overlayOpen && overlayData && overlayData.type !== "we") {
          var key = ImageService.thumbKey(overlayData.thumb, overlayData.name)
          _hexMeta = FileMetadataService.getMetadata(key)
          if (!_hexMeta)
            FileMetadataService.probeIfNeeded(key, overlayData.path, overlayData.type === "video" ? "video" : "image")
        }
      }
      Connections {
        target: FileMetadataService
        enabled: hexBackOverlay.overlayOpen
        function onMetadataReady(key) {
          if (!hexBackOverlay.overlayData) return
          var myKey = ImageService.thumbKey(hexBackOverlay.overlayData.thumb, hexBackOverlay.overlayData.name)
          if (key === myKey)
            hexBackOverlay._hexMeta = FileMetadataService.getMetadata(key)
        }
      }
      readonly property real bigW: bigR * 2
      readonly property real bigH: Math.ceil(bigR * 1.73205)
      readonly property real _cos30: 0.866025
      readonly property real _sin30: 0.5

      function show(data, gx, gy, sourceItem) {
        overlayTagField._syncing = true; overlayTagField.text = ""; overlayTagField._sessionTags = []; overlayTagField._syncing = false
        overlayData = data
        overlayItemKey = (data.weId || "") !== "" ? data.weId : data.name
        _sourceItem = sourceItem || null
        _openContentX = hexListView.contentX
        var local = hexBackOverlay.mapFromItem(null, gx, gy)
        sourceX = local.x
        sourceY = local.y
        visible = true
        overlayOpen = true
      }

      function hide() {
        var scrollDelta = hexListView.contentX - _openContentX
        sourceX -= scrollDelta
        _openContentX = hexListView.contentX
        overlayOpen = false
      }

      Rectangle {
        anchors.fill: parent
        color: Qt.rgba(0, 0, 0, hexBackOverlay.overlayOpen ? 0.55 : 0)
        Behavior on color { ColorAnimation { duration: Style.animNormal } }

        MouseArea {
          anchors.fill: parent
          acceptedButtons: Qt.LeftButton | Qt.RightButton
          onClicked: hexBackOverlay.hide()
        }
      }

      states: [
        State {
          name: "hidden"
          when: !hexBackOverlay.overlayOpen
          PropertyChanges {
            target: hexCard
            x: hexBackOverlay.sourceX - hexCard.width / 2
            y: hexBackOverlay.sourceY - hexCard.height / 2
            scale: wallpaperSelector.hexRadius / hexBackOverlay.bigR
            opacity: 0
          }
          PropertyChanges {
            target: cardRotation
            angle: 0
          }
        },
        State {
          name: "visible"
          when: hexBackOverlay.overlayOpen
          PropertyChanges {
            target: hexCard
            x: (hexBackOverlay.width - hexCard.width) / 2
            y: (hexBackOverlay.height - hexCard.height) / 2
            scale: 1
            opacity: 1
          }
          PropertyChanges {
            target: cardRotation
            angle: 180
          }
        }
      ]

      transitions: [
        Transition {
          from: "hidden"; to: "visible"
          SequentialAnimation {
            PropertyAction { target: hexBackOverlay; property: "visible"; value: true }
            ParallelAnimation {
              NumberAnimation { target: hexCard; properties: "x,y,scale,opacity"; duration: Style.animSlow; easing.type: Easing.OutCubic }
              NumberAnimation { target: cardRotation; property: "angle"; duration: Style.animSlow; easing.type: Easing.InOutQuad }
            }
          }
        },
        Transition {
          from: "visible"; to: "hidden"
          SequentialAnimation {
            ParallelAnimation {
              NumberAnimation { target: hexCard; properties: "x,y,scale"; duration: Style.animSlow; easing.type: Easing.InOutCubic }
              NumberAnimation { target: cardRotation; property: "angle"; duration: Style.animSlow; easing.type: Easing.InOutQuad }
              SequentialAnimation {
                PauseAnimation { duration: Style.animSlow * 0.7 }
                NumberAnimation { target: hexCard; property: "opacity"; duration: Style.animSlow * 0.3; easing.type: Easing.InQuad }
              }
            }
            PropertyAction { target: hexBackOverlay; property: "visible"; value: false }
            PropertyAction { target: hexBackOverlay; property: "overlayItemKey"; value: "" }
            PropertyAction { target: hexBackOverlay; property: "_sourceItem"; value: null }
          }
        }
      ]

      Item {
        id: hexCard
        width: hexBackOverlay.bigW
        height: hexBackOverlay.bigH
        transformOrigin: Item.Center

        transform: Rotation {
          id: cardRotation
          origin.x: hexCard.width / 2
          origin.y: hexCard.height / 2
          axis { x: 0; y: 1; z: 0 }
          angle: 0
        }

        Item {
          id: bigHexMask
          width: hexCard.width; height: hexCard.height
          visible: false
          layer.enabled: true
          Shape {
            anchors.fill: parent; antialiasing: true; preferredRendererType: Shape.CurveRenderer
            ShapePath {
              fillColor: "white"; strokeColor: "transparent"
              startX: hexBackOverlay.bigR * 2;  startY: hexCard.height / 2
              PathLine { x: hexBackOverlay.bigR + hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 - hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: hexBackOverlay.bigR - hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 - hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: 0;                                                                  y: hexCard.height / 2 }
              PathLine { x: hexBackOverlay.bigR - hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 + hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: hexBackOverlay.bigR + hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 + hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: hexBackOverlay.bigR * 2;                                            y: hexCard.height / 2 }
            }
          }
        }

        Item {
          id: frontFace
          anchors.fill: parent
          visible: cardRotation.angle < 90

          Item {
            anchors.fill: parent
            Image {
              anchors.fill: parent
              source: hexBackOverlay.overlayData && hexBackOverlay.overlayData.thumb
                ? ImageService.fileUrl(hexBackOverlay.overlayData.thumb) : ""
              fillMode: Image.PreserveAspectCrop
              smooth: true
              asynchronous: true; cache: false
              sourceSize.width: hexBackOverlay.bigW
              sourceSize.height: hexBackOverlay.bigH
            }
            layer.enabled: true; layer.smooth: true
            layer.effect: MultiEffect { maskEnabled: true; maskSource: bigHexMask; maskThresholdMin: 0.3; maskSpreadAtMin: 0.3 }
          }

          Shape {
            anchors.fill: parent; antialiasing: true; preferredRendererType: Shape.CurveRenderer
            ShapePath {
              fillColor: "transparent"
              strokeColor: wallpaperSelector.colors ? wallpaperSelector.colors.primary : Style.fallbackAccent
              strokeWidth: 2
              startX: hexBackOverlay.bigR * 2;  startY: hexCard.height / 2
              PathLine { x: hexBackOverlay.bigR + hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 - hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: hexBackOverlay.bigR - hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 - hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: 0;                                                                  y: hexCard.height / 2 }
              PathLine { x: hexBackOverlay.bigR - hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 + hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: hexBackOverlay.bigR + hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 + hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: hexBackOverlay.bigR * 2;                                            y: hexCard.height / 2 }
            }
          }

        }

        Item {
          id: backFace
          anchors.fill: parent
          visible: cardRotation.angle >= 90
          transform: Rotation {
            origin.x: backFace.width / 2; origin.y: backFace.height / 2
            axis { x: 0; y: 1; z: 0 }
            angle: 180
          }

          Item {
            id: backClip
            anchors.fill: parent

            MouseArea {
              anchors.fill: parent
              acceptedButtons: Qt.RightButton
              z: -1
              onClicked: hexBackOverlay.hide()
            }

            Rectangle { anchors.fill: parent; color: wallpaperSelector.colors ? wallpaperSelector.colors.surfaceContainer : "#1a1a2e" }

            Image {
              anchors.fill: parent
              source: hexBackOverlay.overlayData && hexBackOverlay.overlayData.thumb
                ? ImageService.fileUrl(hexBackOverlay.overlayData.thumb) : ""
              fillMode: Image.PreserveAspectCrop; opacity: 0.08
              sourceSize.width: 120
              sourceSize.height: 104
              asynchronous: true; cache: false
            }

            Column {
              id: backContent
              anchors.centerIn: parent
              width: hexBackOverlay.bigR * 1.6
              spacing: 4

              Text {
                width: parent.width
                text: hexBackOverlay.overlayData ? hexBackOverlay.overlayData.name.replace(/\.[^/.]+$/, "").toUpperCase() : ""
                color: wallpaperSelector.colors ? wallpaperSelector.colors.tertiary : "#8bceff"
                font.family: Style.fontFamily; font.pixelSize: 15; font.weight: Font.Bold; font.letterSpacing: 1.2
                horizontalAlignment: Text.AlignHCenter; wrapMode: Text.Wrap; elide: Text.ElideRight; maximumLineCount: 2
              }

              Row {
                anchors.horizontalCenter: parent.horizontalCenter
                spacing: 0
                visible: hexBackOverlay.overlayData && hexBackOverlay.overlayData.type !== "we"
                Text {
                  text: hexBackOverlay.overlayData ? FileMetadataService.formatExt(hexBackOverlay.overlayData.name) : ""
                  color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.tertiary.r, wallpaperSelector.colors.tertiary.g, wallpaperSelector.colors.tertiary.b, 0.6) : Qt.rgba(1,1,1,0.35)
                  font.family: Style.fontFamily; font.pixelSize: 11; font.weight: Font.Medium; font.letterSpacing: 0.8
                }
                Text {
                  text: "  \u2022  "; color: Qt.rgba(1,1,1,0.15); font.family: Style.fontFamily; font.pixelSize: 11
                }
                Text {
                  text: hexBackOverlay._hexMeta ? (hexBackOverlay._hexMeta.width + " \u00d7 " + hexBackOverlay._hexMeta.height) : "\u2013"
                  color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.tertiary.r, wallpaperSelector.colors.tertiary.g, wallpaperSelector.colors.tertiary.b, 0.6) : Qt.rgba(1,1,1,0.35)
                  font.family: Style.fontFamily; font.pixelSize: 11; font.weight: Font.Medium; font.letterSpacing: 0.5
                }
                Text {
                  text: "  \u2022  "; color: Qt.rgba(1,1,1,0.15); font.family: Style.fontFamily; font.pixelSize: 11
                }
                Text {
                  text: hexBackOverlay._hexMeta ? FileMetadataService.formatSize(hexBackOverlay._hexMeta.filesize) : "\u2013"
                  color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.tertiary.r, wallpaperSelector.colors.tertiary.g, wallpaperSelector.colors.tertiary.b, 0.6) : Qt.rgba(1,1,1,0.35)
                  font.family: Style.fontFamily; font.pixelSize: 11; font.weight: Font.Medium; font.letterSpacing: 0.5
                }
              }

              Rectangle { width: parent.width; height: 1; color: Qt.rgba(1, 1, 1, 0.08) }

              Item {
                width: parent.width; height: 26
                Text {
                  anchors.left: parent.left; anchors.verticalCenter: parent.verticalCenter
                  text: "FAVOURITE"
                  color: wallpaperSelector.colors ? wallpaperSelector.colors.tertiary : "#8bceff"
                  font.family: Style.fontFamily; font.pixelSize: 12; font.weight: Font.Medium; font.letterSpacing: 0.5
                }
                Item {
                  id: overlayFavToggle
                  anchors.right: parent.right; anchors.verticalCenter: parent.verticalCenter
                  width: 44; height: 22
                  property bool checked: false
                  Connections {
                    target: hexBackOverlay
                    function onOverlayOpenChanged() {
                      if (hexBackOverlay.overlayOpen && hexBackOverlay.overlayData) {
                        var key = (hexBackOverlay.overlayData.weId || "") !== "" ? hexBackOverlay.overlayData.weId : hexBackOverlay.overlayData.name
                        overlayFavToggle.checked = wallpaperSelector.selectorService ? !!wallpaperSelector.selectorService.favouritesDb[key] : false
                      }
                    }
                  }
                  Canvas {
                    anchors.fill: parent
                    property bool isOn: overlayFavToggle.checked
                    property color fillColor: isOn
                      ? (wallpaperSelector.colors ? wallpaperSelector.colors.primary : Style.fallbackAccent)
                      : Qt.rgba(1, 1, 1, 0.15)
                    onFillColorChanged: requestPaint(); onIsOnChanged: requestPaint()
                    onPaint: {
                      var ctx = getContext("2d"); ctx.clearRect(0, 0, width, height)
                      var sk = 6; ctx.fillStyle = fillColor; ctx.beginPath()
                      ctx.moveTo(sk, 0); ctx.lineTo(width, 0); ctx.lineTo(width - sk, height); ctx.lineTo(0, height)
                      ctx.closePath(); ctx.fill()
                    }
                  }
                  Canvas {
                    width: 20; height: 16; y: 3
                    x: overlayFavToggle.checked ? parent.width - width - 3 : 3
                    Behavior on x { NumberAnimation { duration: Style.animFast; easing.type: Easing.OutCubic } }
                    property color knobColor: overlayFavToggle.checked
                      ? (wallpaperSelector.colors ? wallpaperSelector.colors.primaryText : "#000")
                      : (wallpaperSelector.colors ? wallpaperSelector.colors.surfaceText : "#fff")
                    onKnobColorChanged: requestPaint()
                    onPaint: {
                      var ctx = getContext("2d"); ctx.clearRect(0, 0, width, height)
                      var sk = 4; ctx.fillStyle = knobColor; ctx.beginPath()
                      ctx.moveTo(sk, 0); ctx.lineTo(width, 0); ctx.lineTo(width - sk, height); ctx.lineTo(0, height)
                      ctx.closePath(); ctx.fill()
                    }
                  }
                  MouseArea {
                    anchors.fill: parent; cursorShape: Qt.PointingHandCursor
                    onClicked: {
                      if (!hexBackOverlay.overlayData) return
                      overlayFavToggle.checked = !overlayFavToggle.checked
                      wallpaperSelector.selectorService.toggleFavourite(hexBackOverlay.overlayData.name, hexBackOverlay.overlayData.weId || "")
                    }
                  }
                }
              }

              Rectangle { width: parent.width; height: 1; color: Qt.rgba(1, 1, 1, 0.08) }

              Item {
                width: parent.width; height: 24
                Rectangle {
                  anchors.fill: parent
                  color: overlayTagField.activeFocus
                    ? (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.surface.r, wallpaperSelector.colors.surface.g, wallpaperSelector.colors.surface.b, 0.5) : Qt.rgba(0, 0, 0, 0.3))
                    : "transparent"
                  border.width: 1
                  border.color: overlayTagField.activeFocus
                    ? (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.primary.r, wallpaperSelector.colors.primary.g, wallpaperSelector.colors.primary.b, 0.5) : Qt.rgba(1, 1, 1, 0.3))
                    : (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.outline.r, wallpaperSelector.colors.outline.g, wallpaperSelector.colors.outline.b, 0.2) : Qt.rgba(1, 1, 1, 0.1))
                  Behavior on color { ColorAnimation { duration: Style.animVeryFast } }
                  Behavior on border.color { ColorAnimation { duration: Style.animVeryFast } }
                }
                TextInput {
                  id: overlayTagField
                  anchors.fill: parent; anchors.leftMargin: 10; anchors.rightMargin: 10
                  verticalAlignment: TextInput.AlignVCenter
                  font.family: Style.fontFamily; font.pixelSize: 11; font.letterSpacing: 0.3
                  color: wallpaperSelector.colors ? wallpaperSelector.colors.surfaceText : "#fff"
                  clip: true
                  property var _sessionTags: []
                  property bool _syncing: false
                  onTextChanged: {
                    if (_syncing) return
                    if (!hexBackOverlay.overlayData) return
                    var raw = text.toLowerCase()
                    var words = raw.split(/\s+/).filter(function(w) { return w.length > 0 })
                    var wpTags = wallpaperSelector.selectorService.getWallpaperTags(overlayTagsSection.wpName, overlayTagsSection.wpWeId).slice()
                    var changed = false
                    for (var i = 0; i < words.length; i++) {
                      if (_sessionTags.indexOf(words[i]) === -1) _sessionTags.push(words[i])
                      if (wpTags.indexOf(words[i]) === -1) { wpTags.push(words[i]); changed = true }
                    }
                    var toRemove = []
                    for (var k = 0; k < _sessionTags.length; k++) {
                      if (words.indexOf(_sessionTags[k]) === -1) toRemove.push(_sessionTags[k])
                    }
                    for (var r = 0; r < toRemove.length; r++) {
                      var si = _sessionTags.indexOf(toRemove[r])
                      if (si !== -1) _sessionTags.splice(si, 1)
                      var wi = wpTags.indexOf(toRemove[r])
                      if (wi !== -1) { wpTags.splice(wi, 1); changed = true }
                    }
                    if (changed) wallpaperSelector.selectorService.setWallpaperTags(overlayTagsSection.wpName, overlayTagsSection.wpWeId, wpTags)
                  }
                  Keys.onReturnPressed: function(event) { event.accepted = true }
                  Keys.onEscapePressed: { _syncing = true; text = ""; _sessionTags = []; _syncing = false; hexBackOverlay.hide() }
                  Text {
                    anchors.fill: parent; verticalAlignment: Text.AlignVCenter
                    text: "+ ADD TAG"; font.family: Style.fontFamily; font.pixelSize: 11; font.letterSpacing: 1
                    color: wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.surfaceText.r, wallpaperSelector.colors.surfaceText.g, wallpaperSelector.colors.surfaceText.b, 0.25) : Qt.rgba(1, 1, 1, 0.2)
                    visible: !parent.text && !parent.activeFocus
                  }
                }
                MouseArea { anchors.fill: parent; cursorShape: Qt.IBeamCursor; z: -1; onClicked: overlayTagField.forceActiveFocus() }
              }

              Item {
                id: overlayTagsSection
                width: parent.width
                height: Math.min(Math.max(30, overlayTagsFlow.implicitHeight + 10), hexBackOverlay.bigR * 0.5)
                clip: true

                property string wpName: hexBackOverlay.overlayData ? hexBackOverlay.overlayData.name : ""
                property string wpWeId: hexBackOverlay.overlayData ? (hexBackOverlay.overlayData.weId || "") : ""
                property var currentTags: {
                  if (!hexBackOverlay.overlayOpen) return []
                  var db = wallpaperSelector.selectorService ? wallpaperSelector.selectorService.tagsDb : null
                  if (!db) return []
                  var key = overlayTagsSection.wpWeId ? overlayTagsSection.wpWeId : ImageService.thumbKey(hexBackOverlay.overlayData ? hexBackOverlay.overlayData.thumb : "", overlayTagsSection.wpName)
                  return db[key] || []
                }

                Flickable {
                  anchors.fill: parent; contentHeight: overlayTagsFlow.implicitHeight
                  clip: true; flickableDirection: Flickable.VerticalFlick; boundsBehavior: Flickable.StopAtBounds
                  Flow {
                    id: overlayTagsFlow; width: parent.width; spacing: 5
                    Repeater {
                      model: overlayTagsSection.currentTags
                      Rectangle {
                        property bool hovered: _tagMa.containsMouse
                        width: _tagTxt.implicitWidth + 30; height: 28; radius: 4
                        color: hovered ? (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.surfaceVariant.r, wallpaperSelector.colors.surfaceVariant.g, wallpaperSelector.colors.surfaceVariant.b, 0.5) : Qt.rgba(1,1,1,0.15)) : "transparent"
                        border.width: 1
                        border.color: hovered
                          ? (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.primary.r, wallpaperSelector.colors.primary.g, wallpaperSelector.colors.primary.b, 0.7) : Qt.rgba(1,1,1,0.3))
                          : (wallpaperSelector.colors ? Qt.rgba(wallpaperSelector.colors.outline.r, wallpaperSelector.colors.outline.g, wallpaperSelector.colors.outline.b, 0.5) : Qt.rgba(1,1,1,0.15))
                        Behavior on color { ColorAnimation { duration: Style.animVeryFast } }
                        Behavior on border.color { ColorAnimation { duration: Style.animVeryFast } }
                        transform: Matrix4x4 { matrix: Qt.matrix4x4(1, -0.08, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1) }
                        Text {
                          id: _tagTxt; anchors.left: parent.left; anchors.leftMargin: 8; anchors.verticalCenter: parent.verticalCenter
                          text: modelData.toUpperCase(); color: wallpaperSelector.colors ? wallpaperSelector.colors.tertiary : "#8bceff"
                          font.family: Style.fontFamily; font.pixelSize: 12; font.weight: Font.Medium; font.letterSpacing: 0.5
                        }
                        Text {
                          anchors.right: parent.right; anchors.rightMargin: 6; anchors.verticalCenter: parent.verticalCenter
                          text: "\u{f0156}"; font.family: Style.fontFamilyNerdIcons; font.pixelSize: 11
                          color: parent.hovered ? (wallpaperSelector.colors ? wallpaperSelector.colors.primary : "#ff6b6b") : Qt.rgba(1,1,1,0.25)
                          Behavior on color { ColorAnimation { duration: Style.animVeryFast } }
                        }
                        MouseArea {
                          id: _tagMa; anchors.fill: parent; hoverEnabled: true; cursorShape: Qt.PointingHandCursor
                          onClicked: {
                            var tags = wallpaperSelector.selectorService.getWallpaperTags(overlayTagsSection.wpName, overlayTagsSection.wpWeId).slice()
                            var idx = tags.indexOf(modelData); if (idx !== -1) tags.splice(idx, 1)
                            wallpaperSelector.selectorService.setWallpaperTags(overlayTagsSection.wpName, overlayTagsSection.wpWeId, tags)
                          }
                        }
                      }
                    }
                  }
                }
                Text {
                  anchors.centerIn: parent; visible: overlayTagsSection.currentTags.length === 0
                  text: "NO TAGS"; color: Qt.rgba(1,1,1,0.15); font.family: Style.fontFamily; font.pixelSize: 12; font.letterSpacing: 2
                }
              }

              Row {
                id: overlayActionRow
                width: parent.width; height: 32; spacing: 8

                ActionButton {
                  width: hexBackOverlay.overlayData && hexBackOverlay.overlayData.type === "we" ? (parent.width - parent.spacing * 2) / 3 : (parent.width - parent.spacing) / 2
                  colors: wallpaperSelector.colors
                  icon: "\u{f0208}"; label: "VIEW"
                  onClicked: { if (!hexBackOverlay.overlayData) return; var p = hexBackOverlay.overlayData.path; Qt.openUrlExternally(ImageService.fileUrl(p.substring(0, p.lastIndexOf("/")))); hexBackOverlay.hide() }
                }

                ActionButton {
                  width: hexBackOverlay.overlayData && hexBackOverlay.overlayData.type === "we" ? (parent.width - parent.spacing * 2) / 3 : (parent.width - parent.spacing) / 2
                  colors: wallpaperSelector.colors
                  icon: "\u{f0a79}"; label: "DELETE"; danger: true
                  onClicked: { if (!hexBackOverlay.overlayData) return; wallpaperSelector.selectorService.deleteWallpaperItem(hexBackOverlay.overlayData.type, hexBackOverlay.overlayData.name, hexBackOverlay.overlayData.weId || ""); hexBackOverlay.hide() }
                }

                ActionButton {
                  visible: hexBackOverlay.overlayData && hexBackOverlay.overlayData.type === "we"
                  width: visible ? (parent.width - parent.spacing * 2) / 3 : 0
                  colors: wallpaperSelector.colors
                  icon: "\u{f0bef}"; label: "STEAM"
                  onClicked: { wallpaperSelector.selectorService.openSteamPage(hexBackOverlay.overlayData.weId || ""); hexBackOverlay.hide() }
                }
              }
            }

            layer.enabled: true; layer.smooth: true
            layer.effect: MultiEffect { maskEnabled: true; maskSource: bigHexMask; maskThresholdMin: 0.3; maskSpreadAtMin: 0.3 }
          }

          Shape {
            anchors.fill: parent; antialiasing: true; preferredRendererType: Shape.CurveRenderer
            ShapePath {
              fillColor: "transparent"
              strokeColor: wallpaperSelector.colors ? wallpaperSelector.colors.primary : Style.fallbackAccent
              strokeWidth: 2.5
              startX: hexBackOverlay.bigR * 2;  startY: hexCard.height / 2
              PathLine { x: hexBackOverlay.bigR + hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 - hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: hexBackOverlay.bigR - hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 - hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: 0;                                                                  y: hexCard.height / 2 }
              PathLine { x: hexBackOverlay.bigR - hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 + hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: hexBackOverlay.bigR + hexBackOverlay.bigR * hexBackOverlay._sin30; y: hexCard.height / 2 + hexBackOverlay.bigR * hexBackOverlay._cos30 }
              PathLine { x: hexBackOverlay.bigR * 2;                                            y: hexCard.height / 2 }
            }
          }

        }

      }
    }

  }
}
