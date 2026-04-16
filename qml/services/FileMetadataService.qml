pragma Singleton
import QtQuick
import Quickshell.Io
import ".."

QtObject {
    id: service

    property var metadataDb: ({})
    property bool loaded: false

    signal metadataReady(string key)

    function getMetadata(key) {
        return metadataDb[key] || null
    }

    function loadFromDaemonData(wallpapers) {
        var db = {}
        for (var i = 0; i < wallpapers.length; i++) {
            var r = wallpapers[i]
            var key = r.key || r.name
            if (r.filesize || r.width || r.height) {
                db[key] = {
                    filesize: r.filesize || 0,
                    width: r.width || 0,
                    height: r.height || 0
                }
            }
        }
        metadataDb = db
        loaded = true
    }

    function probeIfNeeded(key, path, type) {
        if (!path || metadataDb[key] || _probing[key]) return
        _probing[key] = true
        _queue.push({ key: key, path: path, type: type })
        _startNext()
    }

    function formatSize(bytes) {
        if (bytes >= 1073741824) return (bytes / 1073741824).toFixed(1) + " GB"
        if (bytes >= 1048576) return (bytes / 1048576).toFixed(1) + " MB"
        if (bytes >= 1024) return (bytes / 1024).toFixed(0) + " KB"
        return bytes + " B"
    }

    function formatExt(name) {
        var dot = name.lastIndexOf(".")
        return dot > 0 ? name.substring(dot + 1).toUpperCase() : ""
    }

    property var _probing: ({})
    property var _queue: []
    property bool _running: false
    property var _current: null

    function _startNext() {
        if (_running || _queue.length === 0) return
        _running = true
        _current = _queue.shift()
        var p = _shellQuote(_current.path)
        var cmd
        if (_current.type === "video") {
            cmd = "s=$(stat -c '%s' " + p + " 2>/dev/null) && " +
                  "d=$(ffprobe -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0 " + p + " 2>/dev/null | head -1) && " +
                  "printf '%s\\t%s\\n' \"$s\" \"$(echo \"$d\" | tr ',' $'\\t')\""
        } else {
            cmd = "s=$(stat -c '%s' " + p + " 2>/dev/null) && " +
                  "d=$(magick identify -format $'%w\\t%h' " + _shellQuote(_current.path + "[0]") + " 2>/dev/null) && " +
                  "printf '%s\\t%s\\n' \"$s\" \"$d\""
        }
        _probeProcess.command = ["sh", "-c", cmd]
        _probeProcess.running = true
    }

    function _shellQuote(s) {
        return "'" + String(s).replace(/'/g, "'\\''") + "'"
    }

    property var _probeProcess: Process {
        property string _out: ""
        stdout: SplitParser { onRead: data => _probeProcess._out = data.trim() }
        onStarted: { _out = "" }
        onExited: {
            var cur = service._current
            service._running = false
            service._current = null
            if (_out) {
                var parts = _out.split("\t")
                if (parts.length >= 3) {
                    var meta = {
                        filesize: parseInt(parts[0]) || 0,
                        width: parseInt(parts[1]) || 0,
                        height: parseInt(parts[2]) || 0
                    }
                    var db = service.metadataDb
                    db[cur.key] = meta
                    service.metadataDb = db
                    DaemonClient.updateMetadata(cur.key, meta.filesize, meta.width, meta.height)
                    service.metadataReady(cur.key)
                }
            }
            delete service._probing[cur.key]
            service._startNext()
        }
    }
}
