import QtQuick
import ".."
import "../services"

ActionButton {
    id: root

    property string wpKey: ""
    property bool hasTags: false
    signal retagStarted()

    property bool inFlight: false
    property bool _failed: false
    property int _elapsed: 0

    icon: "\u{f0450}"
    label: inFlight ? ("TAGGING " + _elapsed + "s") : (_failed ? "FAILED" : (hasTags ? "RETAG" : "TAG"))
    danger: _failed
    enabled: !inFlight && Config.ollamaEnabled && wpKey !== ""

    onClicked: {
        if (wpKey === "") return
        root._failed = false
        root.inFlight = true
        root._elapsed = 0
        root.retagStarted()
        DaemonClient.retagOne(wpKey, function(_result, error) {
            if (error) { root.inFlight = false; root._failed = true; _failedClear.restart() }
        })
    }

    Timer {
        running: root.inFlight; repeat: true; interval: 1000
        onTriggered: root._elapsed += 1
    }

    Timer {
        running: root.inFlight; repeat: false; interval: 120000
        onTriggered: { root.inFlight = false; root._failed = true; _failedClear.restart() }
    }

    Timer { id: _failedClear; interval: 4000; onTriggered: root._failed = false }

    Connections {
        target: WallpaperAnalysisService
        enabled: root.inFlight
        function onItemAnalyzed(key, tags, colors, weather) {
            if (key === root.wpKey) root.inFlight = false
        }
        function onItemFailed(key, error) {
            if (key === root.wpKey) { root.inFlight = false; root._failed = true; _failedClear.restart() }
        }
    }
}
