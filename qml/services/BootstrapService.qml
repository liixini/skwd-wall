pragma Singleton
import QtQuick
import Quickshell.Io
import ".."

QtObject {
    id: bootstrap

    readonly property bool ready: _done
    property bool _done: false

    readonly property string _markerFile: Config.configDir + "/.bootstrapped"

    property var _markerCheck: Process {
        id: markerCheck
        onExited: function(code, status) {
            if (code === 0) {
                bootstrap._done = true
            } else {
                retry.start()
            }
        }
    }

    property Timer _retry: Timer {
        id: retry
        interval: 100
        repeat: false
        onTriggered: bootstrap._check()
    }

    function _check() {
        markerCheck.command = ["test", "-f", _markerFile]
        markerCheck.running = true
    }

    Component.onCompleted: _check()
}
