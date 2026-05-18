pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io

QtObject {
    id: client

    readonly property bool connected: _socket.connected
    property bool ready: false

    property bool cacheRunning: false
    property int cacheProgress: 0
    property int cacheTotal: 0

    signal eventReceived(string event, var data)

    signal fileAdded(string name, string path, string type)
    signal fileRemoved(string name, string type)
    signal fileRenamed(string oldName, string newName)
    signal folderRemoved(var names)
    signal weItemAdded(string weId, string weDir)
    signal weItemRemoved(string weId)
    signal scanDone()

    signal cacheReady()
    signal itemCached(var data)

    signal wallpaperApplied(string type, string name, string path, string weId, string key)
    signal wallpaperToggle()
    signal wallpaperShow()
    signal wallpaperHide()

    property bool randomRunning: false
    property int randomInterval: 0
    signal randomStarted(int interval)
    signal randomStopped()

    property int audioCapableCount: 0
    property int audioPlayingCount: 0
    property var audioOutputs: ({})
    property var audioGroups: []

    function refreshAudioState() {
        outputs(function(result, err) {
            if (err || !result) return
            var outs = result.outputs || {}
            var capable = 0
            var playing = 0
            for (var k in outs) {
                var entry = outs[k]
                if (!entry) continue
                var t = entry.type || ""
                if (t === "video" || t === "we") {
                    capable++
                    if (entry.mute === false) playing++
                }
            }
            client.audioOutputs = outs
            client.audioCapableCount = capable
            client.audioPlayingCount = playing
            client._rebuildAudioGroups()
        })
    }

    function _rebuildAudioGroups() {
        var byKey = {}
        var outs = client.audioOutputs || {}
        var weIdSet = {}
        for (var k in outs) {
            var entry = outs[k]
            if (!entry) continue
            var t = entry.type || ""
            if (t !== "video" && t !== "we") continue
            var groupKey
            if (t === "we") {
                groupKey = "we:*"
                if (entry.we_id) weIdSet[entry.we_id] = true
            } else {
                groupKey = "vid:" + (entry.path || "")
            }
            if (!byKey[groupKey]) {
                byKey[groupKey] = {
                    key: groupKey,
                    type: t,
                    path: entry.path || "",
                    weId: entry.we_id || "",
                    name: _audioDisplayName(entry),
                    outputs: [],
                    muted: true,
                    primary: "",
                    volume: 80
                }
            }
            byKey[groupKey].outputs.push(k)
            if (entry.mute === false) {
                byKey[groupKey].muted = false
                if (!byKey[groupKey].primary) byKey[groupKey].primary = k
            }
            if (typeof entry.volume === "number") {
                byKey[groupKey].volume = entry.volume
            }
        }
        var weCount = 0
        for (var w in weIdSet) weCount++
        if (byKey["we:*"] && weCount > 1) {
            byKey["we:*"].name = "Wallpaper Engine · " + weCount + " wallpapers"
        }
        var arr = []
        for (var gk in byKey) {
            var g = byKey[gk]
            g.outputs.sort()
            if (!g.primary) g.primary = g.outputs[0]
            arr.push(g)
        }
        arr.sort(function(a, b) { return (a.name || "").localeCompare(b.name || "") })
        client.audioGroups = arr
    }

    function _audioDisplayName(entry) {
        if ((entry.type || "") === "we") {
            var id = entry.we_id || ""
            return id !== "" ? ("Wallpaper Engine · " + id) : "Wallpaper Engine"
        }
        var p = entry.path || ""
        var idx = p.lastIndexOf("/")
        var name = idx >= 0 ? p.substring(idx + 1) : p
        var dot = name.lastIndexOf(".")
        if (dot > 0) name = name.substring(0, dot)
        return name || "Untitled"
    }

    function call(method, params, callback) {
        if (!_socket.connected) {
            if (callback) callback(null, {code: -1, message: "not connected"})
            return
        }
        var id = _nextId++
        if (callback) _pending[id] = { cb: callback, ts: Date.now() }
        var line = JSON.stringify({method: method, params: params || {}, id: id})
        _socket.write(line + "\n")
        _socket.flush()
    }

    function subscribe(events) { call("subscribe", {events: events}) }
    function status(callback)  { call("status", {}, callback) }

    function toggle() { call("wall.toggle", {}) }
    function show()   { call("wall.show", {}) }
    function hide()   { call("wall.hide", {}) }

    function applyStatic(path, outputs, neighbors, screens, callback) {
        var params = {type: "static", path: path}
        if (outputs && outputs.length > 0) params.outputs = outputs
        if (neighbors && neighbors.length > 0) params.neighbors = neighbors
        if (screens && screens.length > 0) params.screens = screens
        call("wall.apply", params, callback)
    }
    function applyVideo(path, outputs, neighbors, screens, audioMap, volumeMap, callback) {
        var params = {type: "video", path: path}
        if (outputs && outputs.length > 0) params.outputs = outputs
        if (neighbors && neighbors.length > 0) params.neighbors = neighbors
        if (screens && screens.length > 0) params.screens = screens
        if (audioMap) params.outputs_audio = audioMap
        if (volumeMap) params.outputs_volume = volumeMap
        call("wall.apply", params, callback)
    }
    function applyWE(weId, screens, audioMap, volumeMap, callback) {
        var params = {type: "we", we_id: weId, screens: screens || []}
        if (audioMap) params.outputs_audio = audioMap
        if (volumeMap) params.outputs_volume = volumeMap
        call("wall.apply", params, callback)
    }
    function restore(callback) { call("wall.restore", {}, callback) }
    function outputs(callback) { call("wall.outputs", {}, callback) }
    function setAudio(mute, volume, outputs, callback) {
        if (typeof outputs === "function") { callback = outputs; outputs = null }
        let params = {}
        if (mute !== undefined && mute !== null) params.mute = !!mute
        if (volume !== undefined && volume !== null) params.volume = volume | 0
        if (outputs && outputs.length > 0) params.outputs = outputs
        call("wall.set_audio", params, callback)
    }

    function preheat(path) {
        if (!path) return
        call("wall.preheat", {path: path})
    }
    function retheme(scheme, mode, colorIndex, callback) {
        var params = {scheme: scheme || "", mode: mode || ""}
        if (typeof colorIndex === "number") params.color_index = colorIndex | 0
        else if (typeof colorIndex === "function" && callback === undefined) { callback = colorIndex }
        call("wall.retheme", params, callback)
    }
    function themePreview(scheme, mode, colorIndex, callback) {
        call("wall.theme_preview", {
            scheme: scheme || "scheme-fidelity",
            mode: mode || "dark",
            color_index: colorIndex | 0,
        }, callback)
    }

    function rebuildCache(callback)    { call("wall.cache_rebuild", {}, callback) }
    function clearData(callback)       { call("wall.clear_data", {}, callback) }
    function cacheStatus(callback)     { call("wall.cache_status", {}, callback) }

    function listWallpapers(favouritesOnly, callback) {
        call("wall.list", {favourites: !!favouritesOnly}, callback)
    }
    function setFavourite(key, favourite, callback) {
        call("wall.set_favourite", {key: key, favourite: favourite}, callback)
    }
    function updateAnalysis(key, tags, colors, analyzedBy, hue, sat, callback) {
        var params = {key: key}
        if (tags !== undefined && tags !== null) params.tags = JSON.stringify(tags)
        if (colors !== undefined && colors !== null) params.colors = JSON.stringify(colors)
        if (analyzedBy) params.analyzed_by = analyzedBy
        if (hue !== undefined && hue !== null) params.hue = hue
        if (sat !== undefined && sat !== null) params.sat = sat
        call("wall.update_analysis", params, callback)
    }
    function retagOne(key, callback) {
        call("analysis.retag_one", {key: key}, callback)
    }
    function recomputeColors(callback) {
        call("wall.recompute_colors", {}, callback)
    }
    function importFromQml(callback) { call("wall.import", {}, callback) }
    function deleteItem(name, type, weId, callback) {
        var params = {name: name, type: type || "static"}
        if (weId) params.we_id = weId
        call("wall.delete", params, callback)
    }

    function updateMetadata(key, filesize, width, height) {
        call("wall.update_metadata", {key: key, filesize: filesize, width: width, height: height})
    }

    function fetchWeather(callback) {
        call("wall.weather", {}, callback)
    }

    function randomStart(intervalSecs, options, callback) {
        var params = {interval: intervalSecs || 300}
        if (options) {
            if (options.types) params.types = options.types
            if (options.favouritesOnly !== undefined) params.favourites_only = !!options.favouritesOnly
        }
        call("wall.random_start", params, callback)
    }
    function randomStop(callback) {
        call("wall.random_stop", {}, callback)
    }
    function randomStatus(callback) {
        call("wall.random_status", {}, function(result, err) {
            if (!err && result) {
                client.randomRunning = !!result.running
                client.randomInterval = result.interval || 0
            }
            if (callback) callback(result, err)
        })
    }

    function stateGet(key, callback) {
        call("state.get", {key: key}, callback)
    }
    function stateSet(key, value) {
        call("state.set", {key: key, value: value})
    }

    property int _nextId: 1
    property var _pending: ({})

    function _handleLine(line) {
        line = line.trim()
        if (!line) return

        var msg
        try { msg = JSON.parse(line) }
        catch (e) { console.warn("DaemonClient: invalid JSON:", line); return }

        if (msg.event) {
            _handleEvent(msg.event, msg.data || {})
            return
        }

        if (msg.id !== undefined) {
            var entry = _pending[msg.id]
            if (entry) {
                delete _pending[msg.id]
                if (msg.error) entry.cb(null, msg.error)
                else entry.cb(msg.result, null)
            }
        }
    }

    function _handleEvent(event, data) {
        client.eventReceived(event, data)

        switch (event) {
        case "skwd.wall.file_added":
            client.fileAdded(data.name || "", data.path || "", data.type || "static"); break
        case "skwd.wall.file_removed":
            client.fileRemoved(data.name || "", data.type || "static"); break
        case "skwd.wall.file_renamed":
            client.fileRenamed(data.old_name || "", data.new_name || ""); break
        case "skwd.wall.folder_removed":
            client.folderRemoved(data.names || []); break
        case "skwd.wall.we_added":
            client.weItemAdded(data.we_id || "", data.we_dir || ""); break
        case "skwd.wall.we_removed":
            client.weItemRemoved(data.we_id || ""); break
        case "skwd.wall.scan_done":
            client.scanDone(); break
        case "skwd.wall.cache":
            client.cacheRunning = (data.status === "started" || data.status === "progress")
            client.cacheProgress = data.progress || 0
            client.cacheTotal = data.total || 0
            if (data.status === "ready") client.cacheReady()
            break
        case "skwd.wall.cached":
            client.itemCached(data); break
        case "skwd.wall.applied":
            client.wallpaperApplied(data.type || "", data.name || "", data.path || "", data.we_id || "", data.key || "")
            client.refreshAudioState()
            break
        case "skwd.wall.toggle":
            client.wallpaperToggle(); break
        case "skwd.wall.show":
            client.wallpaperShow(); break
        case "skwd.wall.hide":
            client.wallpaperHide(); break
        case "skwd.wall.random_started":
            client.randomRunning = true
            client.randomInterval = data.interval || 0
            client.randomStarted(client.randomInterval); break
        case "skwd.wall.random_stopped":
            client.randomRunning = false
            client.randomInterval = 0
            client.randomStopped(); break
        }
    }

    property var _socket: Socket {
        path: (Quickshell.env("XDG_RUNTIME_DIR") || "/tmp") + "/skwd/daemon.sock"

        connected: false

        parser: SplitParser {
            onRead: data => client._handleLine(data)
        }

        onConnectionStateChanged: {
            if (connected) {
                console.log("DaemonClient: connected")
                client.subscribe(["skwd."])
                client.ready = true
                client.randomStatus()
                client.refreshAudioState()
            } else {
                console.log("DaemonClient: disconnected")
                client.ready = false
                client._pending = {}
                client._reconnectTimer.restart()
            }
        }
    }

    property var _reconnectTimer: Timer {
        interval: 2000
        repeat: false
        onTriggered: {
            if (!client.connected)
                client._socket.connected = true
        }
    }

    property var _cleanupTimer: Timer {
        interval: 30000
        running: true
        repeat: true
        onTriggered: {
            var now = Date.now()
            var stale = []
            for (var id in client._pending) {
                if (now - client._pending[id].ts > 30000) stale.push(id)
            }
            for (var i = 0; i < stale.length; i++) {
                var entry = client._pending[stale[i]]
                delete client._pending[stale[i]]
                if (entry && entry.cb) entry.cb(null, {code: -2, message: "timeout"})
            }
        }
    }

    Component.onCompleted: {
        _socket.connected = true
    }
}
