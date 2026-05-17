pragma Singleton
import QtQuick
import ".."

QtObject {
    id: svc

    property bool busy: false
    property string lastError: ""

    signal previewed(string previewPath)
    signal committed(string finalPath)
    signal discarded()
    signal failed(string error)

    function preview(effect, inputPath, params) {
        if (!inputPath || svc.busy) return
        svc.busy = true
        svc.lastError = ""
        DaemonClient.call("effects.preview", {
            input: inputPath,
            effect: effect,
            params: params || {}
        }, function(result, err) {
            svc.busy = false
            if (err) {
                var msg = (typeof err === "string") ? err : (err.message || JSON.stringify(err))
                svc.lastError = msg
                svc.failed(msg)
            } else {
                svc.previewed((result && result.output) || "")
            }
        })
    }

    function commit(previewPath, inputPath, effect, params) {
        if (!previewPath || !inputPath || svc.busy) return
        svc.busy = true
        svc.lastError = ""
        DaemonClient.call("effects.commit", {
            preview: previewPath,
            input: inputPath,
            effect: effect,
            params: params || {}
        }, function(result, err) {
            svc.busy = false
            if (err) {
                var msg = (typeof err === "string") ? err : (err.message || JSON.stringify(err))
                svc.lastError = msg
                svc.failed(msg)
            } else {
                svc.committed((result && result.output) || "")
            }
        })
    }

    function discard(previewPath) {
        if (!previewPath) return
        DaemonClient.call("effects.discard", { preview: previewPath }, function(result, err) {
            if (!err) svc.discarded()
        })
    }
}
