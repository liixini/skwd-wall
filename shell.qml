import Quickshell
import Quickshell.Wayland
import Quickshell.Io
import QtQuick
import "qml"
import "qml/services"

ShellRoot {
    id: root

    property bool configLoaded: Config.configLoaded
    onConfigLoadedChanged: {
        if (configLoaded) {
            Qt.callLater(function() {
                root._beginSelectorTiming()
                wallpaperSelectorLoader.active = true
            })
        }
    }

    property double selectorOpenRequestedMs: 0
    property bool selectorTimingPending: false
    property string selectorTimingLogFile: Config.cacheDir + "/wallpaper/selector-timing.log"
    property var _timingLogQueue: []

    function _enqueueTimingLine(message) {
        root._timingLogQueue.push(Date.now() + " " + message)
        root._flushTimingLogQueue()
    }

    function _flushTimingLogQueue() {
        if (_timingLogProcess.running || _timingLogQueue.length === 0)
            return

        var line = _timingLogQueue.shift()
        _timingLogProcess.command = [
            "bash", "-lc",
            "mkdir -p " + JSON.stringify(Config.cacheDir + "/wallpaper")
                + " && printf '%s\\n' " + JSON.stringify(line)
                + " >> " + JSON.stringify(selectorTimingLogFile)
        ]
        _timingLogProcess.running = true
    }

    function _beginSelectorTiming() {
        root.selectorOpenRequestedMs = Date.now()
        root.selectorTimingPending = true
        _logWithRam("open requested")
    }

    function _logWithRam(label) {
        _ramProbeLabel = label
        _ramProbe.command = ["bash", "-c",
            "awk '/^(VmSize|VmRSS):/{print $2}' /proc/self/status; " +
            "awk '/^Pss:/{s+=$2} END{print s}' /proc/self/smaps_rollup"
        ]
        _ramProbe.running = true
    }
    property string _ramProbeLabel: ""
    property var _ramProbe: Process {
        id: ramProbe
        property string _stdout: ""
        stdout: SplitParser {
            splitMarker: ""
            onRead: data => ramProbe._stdout += data
        }
        onExited: {
            var lines = ramProbe._stdout.trim().split("\n")
            var vssMb = (parseInt(lines[0]) / 1024).toFixed(1)
            var rssMb = (parseInt(lines[1]) / 1024).toFixed(1)
            var pssMb = (parseInt(lines[2]) / 1024).toFixed(1)
            var msg = "wallpaper-selector timing: " + root._ramProbeLabel
                + " (rss: " + rssMb + " MB, pss: " + pssMb + " MB, vss: " + vssMb + " MB)"
            console.log(msg)
            root._enqueueTimingLine(msg)
            ramProbe._stdout = ""
        }
    }

    Colors {
        id: colors
    }

    Loader {
        id: wallpaperSelectorLoader
        active: false
        source: "qml/wallpaper/WallpaperSelector.qml"
        onLoaded: {
            if (root.selectorTimingPending) {
                var qmlLoadMs = Date.now() - root.selectorOpenRequestedMs
                root._logWithRam("qml loaded in " + qmlLoadMs + " ms")
            }
            item.colors = Qt.binding(() => colors)
            item.showing = true
            item.uiReady.connect(function() {
                if (!root.selectorTimingPending) return
                var elapsed = Date.now() - root.selectorOpenRequestedMs
                var count = item.selectorService ? item.selectorService.filteredModel.count : 0
                root._logWithRam("ready in " + elapsed + " ms (items: " + count + ")")
                root.selectorTimingPending = false
            })
        }
    }

    property var _timingLogProcess: Process {
        id: timingLogProcess
        command: ["bash", "-lc", "true"]
        onExited: root._flushTimingLogQueue()
    }

    Connections {
        target: wallpaperSelectorLoader.item
        function onShowingChanged() {
            if (!wallpaperSelectorLoader.item) return
            if (!wallpaperSelectorLoader.item.showing)
                Qt.quit()
        }
    }

    IpcHandler {
        target: "wallpaper-ui"

        function refresh() {
            if (wallpaperSelectorLoader.item && wallpaperSelectorLoader.item.selectorService)
                wallpaperSelectorLoader.item.selectorService.refreshFromDb()
        }

        function steamUpdate() {
            if (wallpaperSelectorLoader.item && wallpaperSelectorLoader.item.swService)
                wallpaperSelectorLoader.item.swService.refreshDownloadStatus()
        }
    }
}
