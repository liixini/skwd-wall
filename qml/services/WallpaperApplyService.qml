pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import ".."

QtObject {
    id: service

    readonly property string wallpaperDir: Config.wallpaperDir
    readonly property string videoDir: Config.videoDir
    readonly property string weDir: Config.weDir
    readonly property string weAssetsDir: Config.weAssetsDir
    readonly property string cacheDir: Config.cacheDir
    readonly property string mainMonitor: Config.mainMonitor
    readonly property string ollamaUrl: Config.ollamaUrl
    readonly property string ollamaModel: Config.ollamaModel
    property string matugenScheme: "scheme-fidelity"
    property bool wallpaperMute: true
    readonly property string _matugenConfig: cacheDir + "/matugen-config.toml"
    property bool _stateFileLoaded: false
    property var _stateFile: FileView {
        path: service.cacheDir + "/last-wallpaper.json"
        preload: true
        onLoaded: {
            service._stateFileLoaded = true
        }
    }

    Component.onCompleted: {
        var data = Config._data
        if (data.matugen) {
            if (data.matugen.schemeType) matugenScheme = data.matugen.schemeType
        }
        if (data.wallpaperMute !== undefined) wallpaperMute = data.wallpaperMute
    }

    property bool _restoring: false

    signal wallpaperApplied(string type, string name, string path)
    onWallpaperApplied: { _runPostProcessing(type, name, path); _restoring = false }

    function applyStatic(path) {
        console.log("WallpaperApplyService.applyStatic:", path, "wallpaperDir:", wallpaperDir)
        _saveState("static", path, "")
        if (Config.isKDE) {
            awwwProcess.command = ["sh", "-c",
                "pkill mpvpaper 2>/dev/null; " +
                "pkill -9 -f '[l]inux-wallpaperengine' 2>/dev/null; " +
                "pkill awww 2>/dev/null; pkill awww-daemon 2>/dev/null; " +
                "rm -f " + JSON.stringify(videoDir + "/lockscreen-video.mp4") + "; " +
                "plasma-apply-wallpaperimage " + JSON.stringify(path)]
        } else {
            awwwProcess.command = ["sh", "-c",
                "pkill mpvpaper 2>/dev/null; " +
                "pkill -9 -f '[l]inux-wallpaperengine' 2>/dev/null; " +
                "rm -f " + JSON.stringify(videoDir + "/lockscreen-video.mp4") + "; " +
                "if ! pgrep -x awww-daemon >/dev/null; then " +
                "  setsid awww-daemon >/dev/null 2>&1 & disown; " +
                "  for i in 1 2 3 4 5; do sleep 0.3; pgrep -x awww-daemon >/dev/null && break; done; " +
                "fi; " +
                "awww img " + JSON.stringify(path) +
                " --transition-type wipe --transition-angle 45 --transition-duration 0.5"]
        }
        awwwProcess.running = true
        _extractAndTheme(path)
        wallpaperApplied("static", _basename(path), path)
    }

    function applyVideo(path) {
        _saveState("video", path, "")
        if (Config.isKDE) {
            _applyKdeVideo(path)
        } else {
            mpvProcess.command = ["sh", "-c",
                "pkill awww 2>/dev/null; pkill awww-daemon 2>/dev/null; " +
                "pkill mpvpaper 2>/dev/null; " +
                "pkill -9 -f '[l]inux-wallpaperengine' 2>/dev/null; " +
                "rm -f " + JSON.stringify(videoDir + "/lockscreen-video.mp4") + "; " +
                "nohup setsid mpvpaper -o " + (wallpaperMute ? "'loop --mute=yes'" : "'loop'") + " '*' " + JSON.stringify(path) + " </dev/null >/dev/null 2>&1 &"]
            mpvProcess.running = true
        }
        _extractVideoThumb(path)
        wallpaperApplied("video", _basename(path), path)
    }

    function _applyKdeVideo(path) {
        var plugin = Config.kdeVideoPlugin
        var videoUrl = ImageService.fileUrl(path)
        var muteMode = wallpaperMute ? "4" : "0"
        var script =
            "var allDesktops = desktops();" +
            "for (var i = 0; i < allDesktops.length; i++) {" +
            "  var d = allDesktops[i];" +
            "  d.wallpaperPlugin = '" + plugin + "';" +
            "  d.currentConfigGroup = ['Wallpaper', '" + plugin + "', 'General'];" +
            "  d.writeConfig('VideoUrls', '[{\"filename\":\"" + videoUrl + "\",\"enabled\":true}]');" +
            "  d.writeConfig('MuteMode', '" + muteMode + "');" +
            "}"
        mpvProcess.command = ["qdbus6", "org.kde.plasmashell", "/PlasmaShell",
                              "org.kde.PlasmaShell.evaluateScript", script]
        mpvProcess.running = true
    }

    function applyWE(weId) {
        _saveState("we", "", weId)
        _reclaimOllamaVram()
        _pendingAction = function() {
            _launchWE(weId)
            _extractWEThumb(weId)
            wallpaperApplied("we", weId, weDir + "/" + weId)
        }
        _killAll()
    }

    property bool _restoreRequested: false

    function restore() {
        _restoreRequested = true
        _tryRestore()
    }

    function _tryRestore() {
        if (!_restoreRequested || !_stateFileLoaded) return
        _restoreRequested = false
        var text = _stateFile.text().trim()
        if (!text) return
        _restoring = true
        try {
            var state = JSON.parse(text)
            if (state.type === "static" && state.path)
                applyStatic(state.path)
            else if (state.type === "video" && state.path)
                applyVideo(state.path)
            else if (state.type === "we" && state.we_id)
                applyWE(state.we_id)
            else
                _restoring = false
        } catch(e) {
            console.log("WallpaperApplyService: restore failed:", e)
            _restoring = false
        }
    }

    function _saveState(type, path, weId) {
        var obj = { type: type }
        if (path) obj.path = path
        if (weId) obj.we_id = weId
        _stateFile.setText(JSON.stringify(obj))
    }

    property var _pendingAction: null
    property var _killProcess: Process {
        id: killProcess
        onExited: {
            if (service._pendingAction) {
                var action = service._pendingAction
                service._pendingAction = null
                action()
            }
        }
    }

    function _killAll() {
        killProcess.command = ["sh", "-c",
            "pkill -9 -f '[l]inux-wallpaperengine' 2>/dev/null; " +
            "pkill mpvpaper 2>/dev/null; " +
            "pkill awww 2>/dev/null; " +
            "pkill awww-daemon 2>/dev/null; " +
            "rm -f " + JSON.stringify(videoDir + "/lockscreen-video.mp4") + "; " +
            "sleep 2; true"]
        killProcess.running = true
    }

    property var _awwwStderr: []
    property var _awwwProcess: Process {
        id: awwwProcess
        onExited: function(code, status) {
            console.log("WallpaperApplyService: awww exited code=" + code + " status=" + status)
            if (_awwwStderr.length > 0) console.log("WallpaperApplyService: awww stderr:", _awwwStderr.join(""))
            _awwwStderr = []
        }
        stderr: SplitParser { onRead: data => service._awwwStderr.push(data) }
    }
    property var _mpvProcess: Process { id: mpvProcess }
    property var _awwwDaemonProcess: Process { id: awwwDaemonProcess }

    property var _weStderr: []
    property var _weProcess: Process {
        id: weProcess
        onExited: function(code, status) {
            if (code !== 0 || service._weStderr.length > 0) {
                console.log("WallpaperApplyService: WE exited code=" + code + " status=" + status +
                            (service._weStderr.length > 0 ? " stderr: " + service._weStderr.join("") : ""))
                if (code !== 0 && service._pendingWeId) {
                    console.log("WallpaperApplyService: WE scene failed, falling back to preview image")
                    service._applyWePreviewFallback(service._pendingWeId)
                }
            }
            service._weStderr = []
        }
        stderr: SplitParser { onRead: data => service._weStderr.push(data) }
    }

    function _launchWE(weId) {
        _weProjectStdout = []
        _weReadProject.command = ["cat", weDir + "/" + weId + "/project.json"]
        _weReadProject.running = true
        _pendingWeId = weId
    }

    property string _pendingWeId: ""
    property var _weProjectStdout: []
    property var _weReadProject: Process {
        id: weReadProject
        onExited: {
            var text = _weProjectStdout.join("")
            try {
                var proj = JSON.parse(text)
                var weType = (proj.type || "scene").toLowerCase()
                var weFile = proj.file || ""
                var id = service._pendingWeId
                var basePath = service.weDir + "/" + id

                if (weType === "video" && weFile) {
                    var videoPath = basePath + "/" + weFile
                    _symLinkProcess.command = ["ln", "-sf", videoPath,
                                               service.videoDir + "/lockscreen-video.mp4"]
                    _symLinkProcess.running = true
                    if (Config.isKDE) {
                        service._applyKdeVideo(videoPath)
                    } else {
                        var opts = "loop"
                        if (service.wallpaperMute) opts = "loop --mute=yes"
                        weProcess.command = ["sh", "-c",
                            "pkill mpvpaper 2>/dev/null; " +
                            "nohup setsid mpvpaper -o '" + opts + "' '*' " + JSON.stringify(videoPath) + " </dev/null >/dev/null 2>&1 &"]
                        weProcess.running = true
                    }
                } else {
                    _launchWEScene(id)
                }
            } catch(e) {
                service._launchWEScene(service._pendingWeId)
            }
        }
        stdout: SplitParser {
            splitMarker: ""
            onRead: data => _weProjectStdout.push(data)
        }
    }

    property var _symLinkProcess: Process { id: symLinkProcess }

    function _launchWEScene(weId) {
        var mons = Quickshell.screens.map(function(s) { return s.name })
        var screenArgs = ""
        for (var i = 0; i < mons.length; i++)
            screenArgs += " --screen-root " + mons[i] + " --scaling fill"
        var audioFlag = service.wallpaperMute ? "--silent" : ""
        var assetsArg = service.weAssetsDir ? " --assets-dir " + JSON.stringify(service.weAssetsDir) : ""
        var cmd = "nohup setsid linux-wallpaperengine " + audioFlag +
            " --no-fullscreen-pause --noautomute" + screenArgs +
            " --clamp border" +
            assetsArg + " " + JSON.stringify(service.weDir + "/" + weId) +
            " </dev/null >/dev/null 2>&1 &"
        console.log("WallpaperApplyService: launching WE scene:", cmd)
        weProcess.command = ["sh", "-c", cmd]
        weProcess.running = true
    }

    function _applyWePreviewFallback(weId) {
        var basePath = weDir + "/" + weId
        _wePreviewFallbackProc.command = ["sh", "-c",
            "for p in " + JSON.stringify(basePath) + "/preview.jpg " +
            JSON.stringify(basePath) + "/preview.png " +
            JSON.stringify(basePath) + "/preview.gif; do " +
            "[ -f \"$p\" ] && echo \"$p\" && exit 0; done; exit 1"]
        _wePreviewFallbackProc.running = true
    }

    property string _wePreviewStdout: ""
    property var _wePreviewFallbackProc: Process {
        stdout: SplitParser {
            splitMarker: ""
            onRead: data => service._wePreviewStdout += data
        }
        onExited: function(code) {
            var preview = service._wePreviewStdout.trim()
            service._wePreviewStdout = ""
            if (code === 0 && preview) {
                console.log("WallpaperApplyService: applying WE preview fallback:", preview)
                service.applyStatic(preview)
            }
        }
    }

    function _extractAndTheme(path) {
        _copyAndTheme.command = ["sh", "-c",
            "mkdir -p " + JSON.stringify(cacheDir + "/wallpaper") + "; " +
            "cp " + JSON.stringify(path) + " " + JSON.stringify(cacheDir + "/wallpaper/current.jpg") + " 2>/dev/null; " +
            _matugenCmd(path)]
        _copyAndTheme.running = true
    }

    property var _videoThumbStdout: []
    function _extractVideoThumb(videoPath) {
        var name = _basename(videoPath).replace(/\.[^.]+$/, "") + ".jpg"
        var thumbDir = cacheDir + "/wallpaper/video-thumbs"
        var thumbPath = thumbDir + "/" + name
        _videoThumbProcess.command = ["sh", "-c",
            "mkdir -p " + JSON.stringify(thumbDir) + "; " +
            "[ -f " + JSON.stringify(thumbPath) + " ] || " +
            ImageService.videoThumbnailCmd(JSON.stringify(videoPath), JSON.stringify(thumbPath), 0) + "; " +
            "cp " + JSON.stringify(thumbPath) + " " + JSON.stringify(cacheDir + "/wallpaper/current.jpg") + " 2>/dev/null; " +
            _matugenCmd(thumbPath)]
        _videoThumbProcess.running = true
    }
    property var _videoThumbProcess: Process {
        id: videoThumbProcess
        onExited: function(code) {
            if (code === 2) { console.log("WallpaperApplyService: matugen output unchanged, skipping reloads"); return }
            service._propagateColors()
        }
    }

    function _extractWEThumb(weId) {
        _weFindPreviewStdout = []
        _weFindPreview.command = ["find", weDir + "/" + weId, "-maxdepth", "1",
                                  "-iname", "preview.*", "-type", "f"]
        _weFindPreview.running = true
    }

    property var _weFindPreviewStdout: []
    property var _weFindPreview: Process {
        id: weFindPreview
        onExited: {
            var preview = _weFindPreviewStdout.join("").trim().split("\n")[0]
            if (preview) {
                _copyAndTheme.command = ["sh", "-c",
                    "mkdir -p " + JSON.stringify(service.cacheDir + "/wallpaper") + "; " +
                    "cp " + JSON.stringify(preview) + " " + JSON.stringify(service.cacheDir + "/wallpaper/current.jpg") + " 2>/dev/null; " +
                    service._matugenCmd(preview)]
                _copyAndTheme.running = true
            }
        }
        stdout: SplitParser {
            onRead: data => _weFindPreviewStdout.push(data)
        }
    }

    property var _copyAndTheme: Process {
        id: copyAndThemeProcess
        onExited: function(code) {
            if (code === 2) { console.log("WallpaperApplyService: matugen output unchanged, skipping reloads"); return }
            service._propagateColors()
        }
    }

    function _matugenOutputFiles() {
        var ints = Config.integrations
        var files = []
        for (var i = 0; i < ints.length; i++) {
            var o = ints[i].output
            if (!o) continue
            files.push(o.indexOf("/") >= 0 ? Config._resolve(o) : Config.cacheDir + "/" + o)
        }
        return files
    }

    function _matugenCmd(imagePath) {
        if (!Config.matugenEnabled) return "true"
        var outputs = _matugenOutputFiles()
        var hashFiles = outputs.map(function(f) { return JSON.stringify(f) }).join(" ")
        var before = hashFiles ? "_BEFORE=$(md5sum " + hashFiles + " 2>/dev/null | sort); " : ""
        var imgArg = " image -t " + JSON.stringify(matugenScheme) +
            " --source-color-index 0 " + JSON.stringify(imagePath)
        var defaultCfg = Config.defaultMatugenConfig
        var matugen = "command -v matugen >/dev/null && { " +
            "matugen -c " + JSON.stringify(_matugenConfig) + imgArg + "; " +
            (defaultCfg ? "[ -f " + JSON.stringify(defaultCfg) + " ] && matugen -c " + JSON.stringify(defaultCfg) + imgArg + "; " : "") +
            "true; } || true"
        if (!hashFiles) return matugen
        var after = "_AFTER=$(md5sum " + hashFiles + " 2>/dev/null | sort); "
        return before + matugen + "; " + after + '[ "$_BEFORE" = "$_AFTER" ] && exit 2; exit 0'
    }

    function _propagateColors() {
        if (!Config.matugenEnabled) return
        var integrations = Config.integrations
        console.log("propagateColors: running", integrations.length, "integrations")
        for (var i = 0; i < integrations.length; i++) {
            var reload = integrations[i].reload
            if (!reload) continue
            var resolved = Config._resolve(reload)
            if (resolved.indexOf("/") >= 0 && resolved.indexOf(" ") < 0)
                _runReload("sh " + JSON.stringify(resolved))
            else
                _runReload(resolved)
        }

        _runReload("command -v notify-send >/dev/null && notify-send 'Wallpaper Changed' || true")
    }

    function _runReload(cmd) {
        console.log("runReload:", cmd)
        var proc = reloadComponent.createObject(service)
        proc.command = ["sh", "-c", cmd]
        proc.exited.connect(function() { proc.destroy() })
        proc.running = true
    }
    property var reloadComponent: Component {
        Process {}
    }

    function _reclaimOllamaVram() {
        if (!ollamaUrl || !ollamaModel) return
        var xhr = new XMLHttpRequest()
        xhr.open("POST", ollamaUrl + "/api/generate")
        xhr.setRequestHeader("Content-Type", "application/json")
        xhr.send(JSON.stringify({model: ollamaModel, keep_alive: 0}))
    }

    function _runPostProcessing(type, name, path) {
        if (_restoring && !Config.postProcessOnRestore) return
        var cmds = Config.postProcessing
        if (!cmds || cmds.length === 0) return
        for (var i = 0; i < cmds.length; i++) {
            var cmd = cmds[i]
            if (!cmd) continue
            cmd = cmd.replace(/%type%/g, type).replace(/%name%/g, name).replace(/%path%/g, path)
            _runDetached(cmd)
        }
    }

    function _runDetached(cmd) {
        console.log("runDetached:", cmd)
        var proc = reloadComponent.createObject(service)
        proc.command = ["sh", "-c", "nohup setsid sh -c " + JSON.stringify(cmd) + " </dev/null >/dev/null 2>&1 &"]
        proc.exited.connect(function() { proc.destroy() })
        proc.running = true
    }

    function _basename(path) {
        var parts = path.split("/")
        return parts[parts.length - 1]
    }
}
