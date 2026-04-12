pragma Singleton
import QtQuick
import ".."

QtObject {
    id: svc

    readonly property var presets: ({
        "light":    { label: "Light",    quality: 82, formats: ["png", "jpg", "jpeg", "gif"] },
        "balanced": { label: "Balanced", quality: 88, formats: ["png", "jpg", "jpeg", "gif"] },
        "quality":  { label: "Quality",  quality: 94, formats: ["png", "jpg", "jpeg", "gif"] }
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

    signal finished(int optimized, int skippedCount, int failed)

    function optimize(presetKey, resolutionKey) {
        DaemonClient.call("optimize.start", {
            preset: presetKey || "balanced",
            resolution: resolutionKey || "2k"
        })
    }

    function cancel() {
        DaemonClient.call("optimize.cancel", {})
    }

    property var _conn: Connections {
        target: DaemonClient
        function onEventReceived(event, data) {
            if (event === "skwd.wall.optimize.progress") {
                svc.running = data.running || false
                svc.progress = data.progress || 0
                svc.total = data.total || 0
                svc.skipped = data.skipped || 0
                svc.currentFile = data.currentFile || ""
            } else if (event === "skwd.wall.optimize.finished") {
                svc.running = false
                svc.currentFile = ""
                svc.finished(data.optimized || 0, data.skipped || 0, data.failed || 0)
            }
        }
    }
}
