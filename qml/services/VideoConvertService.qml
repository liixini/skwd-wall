pragma Singleton
import QtQuick
import ".."

QtObject {
    id: svc

    readonly property var presets: ({
        "light":    { label: "Light",    crf: 28, maxrate: "6M",  bufsize: "12M" },
        "balanced": { label: "Balanced", crf: 26, maxrate: "10M", bufsize: "20M" },
        "quality":  { label: "Quality",  crf: 23, maxrate: "16M", bufsize: "32M" }
    })

    readonly property var resolutions: ({
        "1080p": { label: "1080p", maxW: 1920, maxH: 1080 },
        "2k":    { label: "2K",    maxW: 2560, maxH: 1440 },
        "4k":    { label: "4K",    maxW: 3840, maxH: 2160 }
    })

    property bool running: false
    property int progress: 0
    property int total: 0
    property int skipped: 0
    property string currentFile: ""

    signal finished(int converted, int skippedCount, int failed)

    function convert(presetKey, resolutionKey) {
        DaemonClient.call("video_convert.start", {
            preset: presetKey || "balanced",
            resolution: resolutionKey || "2k"
        })
    }

    function cancel() {
        DaemonClient.call("video_convert.cancel", {})
    }

    property var _conn: Connections {
        target: DaemonClient
        function onEventReceived(event, data) {
            if (event === "skwd.wall.convert.progress") {
                svc.running = data.running || false
                svc.progress = data.progress || 0
                svc.total = data.total || 0
                svc.skipped = data.skipped || 0
                svc.currentFile = data.currentFile || ""
            } else if (event === "skwd.wall.convert.finished") {
                svc.running = false
                svc.currentFile = ""
                svc.finished(data.converted || 0, data.skipped || 0, data.failed || 0)
            }
        }
    }
}
