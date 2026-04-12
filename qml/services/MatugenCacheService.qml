pragma Singleton
import QtQuick
import ".."

QtObject {
    id: service

    property bool running: false
    property int progress: 0
    property int total: 0

    signal cacheReady()
    signal oneReady(string key, var colors)

    function processOne(path, key) {
        DaemonClient.call("matugen.process_one", { path: path, key: key })
    }

    function removeOne(key) {
    }

    function rebuild() {
        DaemonClient.call("matugen.start", {})
    }

    function rebuildWithCache(existingCache) {
        DaemonClient.call("matugen.start", {})
    }

    property var _conn: Connections {
        target: DaemonClient
        function onEventReceived(event, data) {
            if (event === "skwd.wall.matugen.progress") {
                service.running = data.running || false
                service.progress = data.progress || 0
                service.total = data.total || 0
            } else if (event === "skwd.wall.matugen.ready") {
                service.running = false
                service.cacheReady()
            } else if (event === "skwd.wall.matugen.one") {
                service.oneReady(data.key || "", data.colors || {})
            }
        }
    }
}
