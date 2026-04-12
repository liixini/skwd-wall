import Quickshell
import Quickshell.Io
import QtQuick
import ".."
import "../services"

QtObject {
  id: service


  required property string scriptsDir
  required property string homeDir
  required property string wallpaperDir
  required property string videoDir
  required property string cacheBaseDir
  required property string weDir
  required property string weAssetsDir
  required property bool showing

  property string thumbsCacheDir: cacheBaseDir + "/wallpaper/thumbs"
  property string weCache: cacheBaseDir + "/wallpaper/we-thumbs"

  property bool cacheReady: false
  property string cacheResult: ""
  readonly property int cacheProgress: DaemonClient.cacheProgress
  readonly property int cacheTotal: DaemonClient.cacheTotal
  property bool cacheLoading: DaemonClient.cacheRunning

  property int selectedColorFilter: -1
  property string selectedTypeFilter: ""
  property string sortMode: "color"
  property var selectedTags: []
  property int selectedTagIndex: -1
  property var popularTags: []
  property bool weatherFilterActive: false
  property var currentWeather: []
  property string weatherLocale: ""

  property var tagsDb: ({})
  property var colorsDb: ({})
  property var matugenDb: ({})
  property var weatherDb: ({})
  property var favouritesDb: ({})
  property bool favouriteFilterActive: false
  property bool _favouritesLoaded: false

  function _loadFromDaemon() {
    console.log("[WSS] _loadFromDaemon called, DaemonClient.ready=" + DaemonClient.ready)
    DaemonClient.listWallpapers(false, function(result, error) {
      if (error) {
        console.warn("[WSS] listWallpapers error:", error.message)
        return
      }
      var walls = result.wallpapers || []
      console.log("[WSS] listWallpapers returned " + walls.length + " items")
      var items = []
      var newTags = {}, newColors = {}, newMatugen = {}, newFavs = {}, newWeather = {}

      for (var i = 0; i < walls.length; i++) {
        var r = walls[i]
        var type = r.type, name = r.name, thumb = r.thumb
        if (!name || !thumb) continue
        var key = r.key || name
        var videoFile = r.video_file || "", weId = r.we_id || ""
        var mtime = r.mtime || 0
        var hue = r.hue != null ? r.hue : 99, sat = r.sat || 0

        items.push({
          name: name, type: type, thumb: thumb,
          path: type === "static" ? service.wallpaperDir + "/" + name
              : (type === "video" ? (videoFile || service.videoDir + "/" + name) : ""),
          weId: weId, videoFile: videoFile,
          mtime: mtime, hue: hue, saturation: sat,
          placeholder: false
        })

        if (r.tags) try { newTags[key] = JSON.parse(r.tags) } catch(e) {}
        if (r.colors) try { newColors[key] = JSON.parse(r.colors) } catch(e) {}
        if (r.matugen) try { newMatugen[key] = JSON.parse(r.matugen) } catch(e) {}
        if (r.weather) try { newWeather[key] = JSON.parse(r.weather) } catch(e) {}
        if (r.favourite === 1) newFavs[key] = true
      }

      console.log("[WSS] built " + items.length + " model items from " + walls.length + " daemon rows")
      _wallpaperData = items
      var keys = {}
      for (var k = 0; k < items.length; k++) {
        var lookupKey = items[k].weId || items[k].name
        keys[lookupKey] = true
      }
      _wallpaperDataKeys = keys
      tagsDb = newTags
      colorsDb = newColors
      matugenDb = newMatugen
      weatherDb = newWeather
      if (!_favouritesLoaded) {
        favouritesDb = newFavs
        _favouritesLoaded = true
      }

      cacheReady = true
      cacheResult = "cached"

      _rebuildPopularTags()
      updateFilteredModel()

      if (Config.matugenEnabled)
        MatugenCacheService.rebuildWithCache(matugenDb)
    })
  }

  function refreshFromDb() {
    _loadFromDaemon()
  }

  function startCacheCheck() {
    console.log("[WSS] startCacheCheck called, DaemonClient.ready=" + DaemonClient.ready)
    ollamaTaggingActive = false
    ollamaColorsActive = false
    ollamaEta = ""
    ollamaLogLine = ""
    cacheResult = ""

    if (DaemonClient.ready) {
      _loadFromDaemon()
    }
  }

  property var _daemonConn: Connections {
    target: DaemonClient
    function onReadyChanged() {
      console.log("[WSS] onReadyChanged: ready=" + DaemonClient.ready)
      if (DaemonClient.ready)
        service._loadFromDaemon()
    }
    function onCacheReady() {
      console.log("[WSS] onCacheReady")
      service._loadFromDaemon()
    }
    function onScanDone() {
      console.log("[WSS] onScanDone")
    }
    function onWeItemAdded(weId, weDir) {
      console.log("[WSS] onWeItemAdded: " + weId + " dir=" + weDir)
    }

    function onItemCached(data) {
      var type = data.type || "static"
      var name = data.name || ""
      var thumb = data.thumb || ""
      if (!name || !thumb) return

      var key = data.key || name
      var weId = data.we_id || ""
      var videoFile = data.video_file || ""
      var mtime = data.mtime || 0
      var hue = data.hue != null ? data.hue : 99
      var sat = data.sat || 0

      var lookupKey = weId || name
      if (_wallpaperDataKeys[lookupKey]) return  // already in model

      var item = {
        name: name, type: type, thumb: thumb,
        path: type === "static" ? service.wallpaperDir + "/" + name
            : (type === "video" ? service.videoDir + "/" + name : ""),
        weId: weId, videoFile: videoFile,
        mtime: mtime, hue: hue, saturation: sat,
        placeholder: false
      }
      _wallpaperData.push(item)
      _wallpaperDataKeys[lookupKey] = true

      var idx = _findInsertIndex(item)
      service.filteredModel.insert(idx, item)
    }

    function _findInsertIndex(item) {
      var count = service.filteredModel.count
      for (var i = 0; i < count; i++) {
        var existing = service.filteredModel.get(i)
        if (sortMode === "date") {
          if (item.mtime > existing.mtime) return i
        } else {
          var hueNew = item.hue === 99 ? 100 : item.hue
          var hueEx = existing.hue === 99 ? 100 : existing.hue
          if (hueNew < hueEx) return i
          if (hueNew === hueEx && item.saturation > existing.saturation) return i
        }
      }
      return count
    }

    function onFileAdded(name, path, type) {
      console.log("[WSS] onFileAdded: " + name + " path=" + path + " type=" + type)
    }

    function onFileRenamed(oldName, newName) {
      console.log("[WSS] onFileRenamed: " + oldName + " -> " + newName)

      for (var i = 0; i < service.filteredModel.count; i++) {
        if (service.filteredModel.get(i).name === oldName) {
          service.filteredModel.setProperty(i, "name", newName)
          service.filteredModel.setProperty(i, "path", service.wallpaperDir + "/" + newName)
          break
        }
      }

      for (var j = 0; j < _wallpaperData.length; j++) {
        if (_wallpaperData[j].name === oldName) {
          _wallpaperData[j].name = newName
          _wallpaperData[j].path = service.wallpaperDir + "/" + newName
          break
        }
      }

      delete _wallpaperDataKeys[oldName]
      _wallpaperDataKeys[newName] = true
    }

    function onFileRemoved(name, type) {
      console.log("[WSS] onFileRemoved: " + name + " type=" + type)

      for (var i = service.filteredModel.count - 1; i >= 0; i--) {
        var fi = service.filteredModel.get(i)
        if (fi.name === name) {
          service.filteredModel.remove(i)
          break
        }
      }

      for (var j = _wallpaperData.length - 1; j >= 0; j--) {
        if (_wallpaperData[j].name === name) {
          _wallpaperData.splice(j, 1)
          break
        }
      }

      delete _wallpaperDataKeys[name]
    }

    function onFolderRemoved(names) {
      var nameSet = {}
      for (var n = 0; n < names.length; n++) nameSet[names[n]] = true

      for (var i = service.filteredModel.count - 1; i >= 0; i--) {
        if (nameSet[service.filteredModel.get(i).name])
          service.filteredModel.remove(i)
      }

      for (var j = _wallpaperData.length - 1; j >= 0; j--) {
        var nm = _wallpaperData[j].name
        if (nameSet[nm]) {
          _wallpaperData.splice(j, 1)
          delete _wallpaperDataKeys[nm]
        }
      }
    }
  }

  function isFavourite(name, weId) {
    var key = weId ? weId : name
    return !!favouritesDb[key]
  }

  function toggleFavourite(name, weId) {
    var key = weId ? weId : name
    var db = JSON.parse(JSON.stringify(favouritesDb))
    if (db[key]) {
      delete db[key]
    } else {
      db[key] = true
    }
    favouritesDb = db
    DaemonClient.setFavourite(key, !!db[key])
    if (favouriteFilterActive) updateFilteredModel()
  }

  function getWallpaperTags(name, weId) {
    if (weId) return tagsDb[weId] || []
    return tagsDb[name] || []
  }

  function setWallpaperTags(name, weId, tags) {
    var key = weId ? weId : name
    var db = JSON.parse(JSON.stringify(tagsDb))
    db[key] = tags
    tagsDb = db
    _rebuildPopularTags()
    DaemonClient.updateAnalysis(key, tags, null, null, null, null)
    if (selectedTags.length > 0) updateFilteredModel()
  }

  function _rebuildPopularTags() {
    var tagCounts = {}
    var dbSize = 0
    for (var name in tagsDb) {
      dbSize++
      var tags = tagsDb[name]
      for (var i = 0; i < tags.length; i++) {
        tagCounts[tags[i]] = (tagCounts[tags[i]] || 0) + 1
      }
    }
    var tagArray = []
    for (var t in tagCounts) tagArray.push({tag: t, count: tagCounts[t]})
    tagArray.sort(function(a, b) { return b.count - a.count })
    console.log("[WSS] _rebuildPopularTags: tagsDb has " + dbSize + " entries, built " + tagArray.length + " popular tags")
    popularTags = tagArray
  }

  onFavouriteFilterActiveChanged: _debouncedUpdate.restart()

  onWeatherFilterActiveChanged: {
    if (weatherFilterActive && currentWeather.length === 0) {
      _fetchWeather()
    } else {
      _debouncedUpdate.restart()
    }
  }

  function _fetchWeather() {
    DaemonClient.fetchWeather(function(result, error) {
      if (error) {
        console.warn("[WSS] weather fetch error:", error.message)
        weatherFilterActive = false
        return
      }
      currentWeather = result.conditions || []
      weatherLocale = result.locale || ""
      console.log("[WSS] weather conditions:", JSON.stringify(currentWeather), "for", weatherLocale)
      updateFilteredModel()
    })
  }

  property bool ollamaTaggingActive: false
  property bool ollamaColorsActive: false
  property bool ollamaActive: ollamaTaggingActive || ollamaColorsActive
  property int ollamaTotalThumbs: 0
  property int ollamaTaggedCount: 0
  property int ollamaColoredCount: 0
  property string ollamaEta: ""
  property string ollamaLogLine: ""

  property var _wallpaperData: []
  property var _wallpaperDataKeys: ({})
  property var filteredModel: ListModel {}

  signal modelUpdated()
  signal wallpaperApplied()

  function updateFilteredModel(skipCrossfade) {
    _skipCrossfade = !!skipCrossfade

    var items = []
    for (var i = 0; i < _wallpaperData.length; i++) {
      var item = _wallpaperData[i]
      var lookupKey = item.weId ? item.weId : ImageService.thumbKey(item.thumb)
      var ollamaColor = colorsDb[lookupKey]
      var useOllama = Config.colorSource === "ollama" && ollamaColor
      var hue = useOllama ? ollamaColor.hue : item.hue
      var saturation = useOllama ? (ollamaColor.saturation || 0) : (item.saturation || 0)
      var effectiveType = (item.type === "we" && item.videoFile) ? "video" : item.type
      if (selectedTypeFilter !== "" && effectiveType !== selectedTypeFilter) continue
      if (selectedColorFilter !== -1 && hue !== selectedColorFilter) continue
      if (favouriteFilterActive && !isFavourite(item.name, item.weId)) continue

      if (selectedTags.length > 0) {
        var wallpaperTags = tagsDb[lookupKey]
        if (!wallpaperTags) continue
        var allTagsMatch = true
        for (var t = 0; t < selectedTags.length; t++) {
          if (wallpaperTags.indexOf(selectedTags[t]) === -1) { allTagsMatch = false; break }
        }
        if (!allTagsMatch) continue
      }

      if (weatherFilterActive && currentWeather.length > 0) {
        var wpWeather = weatherDb[lookupKey]
        if (!wpWeather || wpWeather.length === 0) continue
        var weatherMatch = false
        for (var w = 0; w < currentWeather.length; w++) {
          if (wpWeather.indexOf(currentWeather[w]) !== -1) { weatherMatch = true; break }
        }
        if (!weatherMatch) continue
      }

      items.push({
        name: item.name, type: item.type, thumb: item.thumb, path: item.path,
        weId: item.weId, videoFile: item.videoFile, mtime: item.mtime,
        hue: hue, saturation: saturation,
        placeholder: !!item.placeholder
      })
    }

    if (sortMode === "date") {
      items.sort(function(a, b) { return b.mtime - a.mtime })
    } else {
      items.sort(function(a, b) {
        var hueA = a.hue === 99 ? 100 : a.hue
        var hueB = b.hue === 99 ? 100 : b.hue
        if (hueA !== hueB) return hueA - hueB
        return b.saturation - a.saturation
      })
    }

    _pendingItems = items
    requestFilterUpdate()
  }

  signal requestFilterUpdate()
  property var _pendingItems: []
  property bool filterTransitioning: false
  property bool _skipCrossfade: false

  function commitFilteredModel() {
    filteredModel.clear()
    if (_pendingItems.length > 0) filteredModel.append(_pendingItems)
    _pendingItems = []
    modelUpdated()
  }

  onSelectedColorFilterChanged: _debouncedUpdate.restart()

  property var _debouncedUpdate: Timer {
    interval: 0
    onTriggered: service.updateFilteredModel()
  }
  onSelectedTypeFilterChanged: updateFilteredModel()

  function applyStatic(path) {
    DaemonClient.applyStatic(path)
    service.wallpaperApplied()
  }

  function applyWE(id) {
    var screens = Quickshell.screens.map(function(s) { return s.name })
    DaemonClient.applyWE(id, screens)
  }

  function applyVideo(path) {
    DaemonClient.applyVideo(path)
  }

  function deleteWallpaperItem(type, name, weId) {
    for (var i = filteredModel.count - 1; i >= 0; i--) {
      var fi = filteredModel.get(i)
      if (fi.name === name && (fi.weId || "") === (weId || "")) {
        filteredModel.remove(i)
        break
      }
    }

    for (var j = _wallpaperData.length - 1; j >= 0; j--) {
      var wi = _wallpaperData[j]
      if (wi.name === name && (wi.weId || "") === (weId || "")) {
        _wallpaperData.splice(j, 1)
        _wallpaperData = _wallpaperData
        break
      }
    }

    DaemonClient.deleteItem(name, type, weId || "")
  }

  function openSteamPage(weId) {
    _unsubscribeWE.command = ["xdg-open", "steam://url/CommunityFilePage/" + weId]
    _unsubscribeWE.running = true
  }

  function clearData() {
    cacheReady = false
    cacheResult = ""
    _wallpaperData = []
    _wallpaperDataKeys = {}
    updateFilteredModel()
    DaemonClient.clearData()
  }

  property var _clearCache: Process {
    id: clearCache
    command: ["bash", "-c", "true"]
    onExited: {
      service.cacheReady = false
      service._wallpaperData = []
    }
  }

  property var _unsubscribeWE: Process { command: ["bash", "-c", "true"] }

  property var _analysisConn: Connections {
    target: WallpaperAnalysisService
    function onProgressUpdated() {
      service.ollamaTaggingActive = WallpaperAnalysisService.running
      service.ollamaColorsActive = WallpaperAnalysisService.running
      service.ollamaTotalThumbs = WallpaperAnalysisService.totalThumbs
      service.ollamaTaggedCount = WallpaperAnalysisService.taggedCount
      service.ollamaColoredCount = WallpaperAnalysisService.coloredCount
      service.ollamaLogLine = WallpaperAnalysisService.lastLog
      service.ollamaEta = WallpaperAnalysisService.eta
      if (!WallpaperAnalysisService.running && WallpaperAnalysisService.lastLog)
        _ollamaErrorClearTimer.restart()
    }
    function onItemAnalyzed(key, tags, colors, weather) {
      service.tagsDb[key] = tags
      service.colorsDb[key] = colors
      if (weather && weather.length > 0) service.weatherDb[key] = weather
      service._analysisItemsDirty = true
    }
    function onAnalysisComplete() {
      service.ollamaTaggingActive = false
      service.ollamaColorsActive = false
      service.ollamaEta = ""
      service.ollamaLogLine = ""
      if (service._analysisItemsDirty) {
        service._analysisItemsDirty = false
        service.tagsDb = service.tagsDb
        service.colorsDb = service.colorsDb
        service._rebuildPopularTags()
        service.updateFilteredModel()
      }
    }
  }

  property bool _analysisItemsDirty: false

  property var _ollamaErrorClearTimer: Timer {
    interval: 5000
    onTriggered: service.ollamaLogLine = ""
  }

  property var _optimizeConn: Connections {
    target: ImageOptimizeService
    function onFinished(optimized, skipped, failed) {
      if (optimized > 0)
        service.refreshFromDb()
    }
  }

  property var _videoConvertConn: Connections {
    target: VideoConvertService
  }

  property var _liveReloadTimer: Timer {
    interval: 30000
    running: service.showing && service.ollamaActive
    repeat: true
    onTriggered: {
      if (service._analysisItemsDirty) {
        service._analysisItemsDirty = false
        service.tagsDb = service.tagsDb
        service.colorsDb = service.colorsDb
        service._rebuildPopularTags()
        service.updateFilteredModel()
      }
    }
  }
}
