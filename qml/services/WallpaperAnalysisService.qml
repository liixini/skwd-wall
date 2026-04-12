pragma Singleton
import QtQuick
import ".."

QtObject {
    id: service

    property bool running: false
    property int progress: 0
    property int total: 0
    property int taggedCount: 0
    property int coloredCount: 0
    property int totalThumbs: 0
    property string lastLog: ""
    property string eta: ""

    signal analysisComplete()
    signal progressUpdated()
    signal itemAnalyzed(string key, var tags, var colors, var weather)

    function start() {
        DaemonClient.call("analysis.start", {})
    }

    function stop() {
        DaemonClient.call("analysis.stop", {})
    }

    function regenerate() {
        DaemonClient.call("analysis.regenerate", {})
    }

    function consolidate() {
        DaemonClient.call("analysis.consolidate", {})
    }

    property var _conn: Connections {
        target: DaemonClient
        function onEventReceived(event, data) {
            if (event === "skwd.wall.analysis.progress") {
                service.running = data.running || false
                service.progress = data.progress || 0
                service.total = data.total || 0
                service.taggedCount = data.taggedCount || 0
                service.coloredCount = data.coloredCount || 0
                service.totalThumbs = data.totalThumbs || 0
                service.lastLog = data.lastLog || ""
                service.eta = data.eta || ""
                service.progressUpdated()
            } else if (event === "skwd.wall.analysis.item") {
                var tags = data.tags || []
                var colors = { hue: data.hue || 99, saturation: data.sat || 0 }
                var weather = data.weather || []
                service.itemAnalyzed(data.key || "", tags, colors, weather)
            } else if (event === "skwd.wall.analysis.complete") {
                service.running = false
                service.eta = ""
                service.lastLog = "Done!"
                service.analysisComplete()
            }
        }
    }
}
