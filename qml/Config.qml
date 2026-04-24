pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io
import "services"

QtObject {
    id: config

    readonly property string version: "0.1.0"

    function _resolve(path) { return path ? path.replace("~", homeDir) : "" }

    readonly property string homeDir: Quickshell.env("HOME")
    readonly property string configDir: Quickshell.env("SKWD_WALL_CONFIG")
        || (Quickshell.env("XDG_CONFIG_HOME") || (homeDir + "/.config")) + "/skwd-wall"
    readonly property string installDir: Quickshell.env("SKWD_WALL_INSTALL")
        || configDir

    property var _configFile: FileView {
        path: BootstrapService.ready ? (configDir + "/config.json") : ""
        watchChanges: true
        onFileChanged: _configFile.reload()
    }
    property string _rawText: _configFile.__text ?? ""
    readonly property bool configLoaded: _rawText !== ""
    property var _data: {
        var raw = _rawText
        if (!raw) return {}
        try { return JSON.parse(raw) }
        catch (e) { return {} }
    }

    function saveKey(path, value) {
        _configWriter.reload()
        var data
        try { data = JSON.parse(_configWriter.text()) } catch(e) { data = {} }
        var parts = path.split(".")
        var obj = data
        for (var i = 0; i < parts.length - 1; i++) {
            if (typeof obj[parts[i]] !== "object" || obj[parts[i]] === null)
                obj[parts[i]] = {}
            obj = obj[parts[i]]
        }
        obj[parts[parts.length - 1]] = value
        _configWriter.setText(JSON.stringify(data, null, 2) + "\n")
    }

    property var _configWriter: FileView {
        path: Config.configDir + "/config.json"
        preload: true
    }

    readonly property string runtimeDir: (Quickshell.env("XDG_RUNTIME_DIR") || "/tmp") + "/skwd-wall"

    readonly property string scriptsDir: _resolve(_data.paths?.scripts) || (installDir + "/scripts")
    readonly property string templateDir: _resolve(_data.paths?.templates) || (installDir + "/data/matugen/templates")
    readonly property string cacheDir: _resolve(_data.paths?.cache)
        || Quickshell.env("SKWD_WALL_CACHE")
        || (Quickshell.env("XDG_CACHE_HOME") || (homeDir + "/.cache")) + "/skwd-wall"
    readonly property string wallpaperDir: _resolve(_data.paths?.wallpaper)
        || (homeDir + "/Pictures/Wallpapers")
    readonly property string videoDir: _resolve(_data.paths?.videoWallpaper)
        || wallpaperDir
    readonly property string weDir: _resolve(_data.paths?.steamWorkshop)
        || _detectWeDir()
    function _detectWeDir() {
        var steamRoot = _resolve(_data.paths?.steam) || (homeDir + "/.local/share/Steam")
        var candidate = steamRoot + "/steamapps/workshop/content/431960"
        return candidate
    }
    readonly property string weAssetsDir: _resolve(_data.paths?.steamWeAssets)
    readonly property string steamDir: _resolve(_data.paths?.steam)

    readonly property string mainMonitor: _data.monitor ?? ""
    readonly property string ollamaUrl: Quickshell.env("SKWD_OLLAMA_URL") || (_data.ollama?.url ?? "")
    readonly property string ollamaModel: _data.ollama?.model ?? ""
    readonly property string ollamaConsolidationModel: _data.ollama?.consolidationModel ?? ""
    readonly property bool ollamaConsolidateEnabled: _data.ollama?.consolidateEnabled !== false

    readonly property string locale: _data.general?.locale ?? ""
    readonly property bool closeOnSelection: _data.general?.closeOnSelection === true
    readonly property bool reopenAtLastSelection: _data.general?.reopenAtLastSelection === true
    readonly property bool filterBarAlwaysVisible: _data.general?.filterBarAlwaysVisible !== false
    readonly property bool searchBarAlwaysVisible: _data.general?.searchBarAlwaysVisible === true
    readonly property int randomInterval: _data.general?.randomInterval ?? 300
    readonly property bool randomIncludeStatic: _data.general?.randomIncludeStatic !== false
    readonly property bool randomIncludeVideo: _data.general?.randomIncludeVideo !== false
    readonly property bool randomIncludeWE: _data.general?.randomIncludeWE !== false
    readonly property bool randomIncludeFavourites: _data.general?.randomIncludeFavourites !== false
    readonly property bool wallpaperPerMonitor: _data.general?.wallpaperPerMonitor === true
    readonly property bool notifyOnWallpaperChange: _data.general?.notifyOnWallpaperChange !== false
    readonly property string notificationsBuiltIn: _data.notifications?.builtIn ?? "never"
    readonly property real uiScale: Math.max(1.0, Math.min(2.0, _data.general?.uiScale ?? 1.0))

    readonly property bool matugenEnabled: _data.features?.matugen !== false
    readonly property bool ollamaEnabled: _data.features?.ollama !== false
    readonly property bool steamEnabled: _data.features?.steam !== false
    readonly property bool wallhavenEnabled: _data.features?.wallhaven !== false
    readonly property bool videoPreviewEnabled: _data.features?.videoPreview !== false
    readonly property bool videoAutoScale: _data.features?.videoAutoScale === true
    readonly property bool wallpaperMute: _data.wallpaperMute !== false
    readonly property int wallpaperVolume: {
        var v = _data.wallpaperVolume
        if (typeof v !== "number") return 100
        return Math.max(0, Math.min(100, Math.round(v)))
    }

    readonly property string videoConvertPreset: _data.performance?.videoConvertPreset ?? "balanced"
    readonly property string videoConvertResolution: _data.performance?.videoConvertResolution ?? "2k"
    readonly property string imageOptimizePreset: _data.performance?.imageOptimizePreset ?? "balanced"
    readonly property string imageOptimizeResolution: _data.performance?.imageOptimizeResolution ?? "2k"

    readonly property bool autoOptimizeImages: _data.performance?.autoOptimizeImages === true
    readonly property bool autoConvertVideos: _data.performance?.autoConvertVideos === true
    readonly property int imageTrashDays: _data.performance?.imageTrashDays ?? 7
    readonly property int videoTrashDays: _data.performance?.videoTrashDays ?? 7
    readonly property bool autoDeleteImageTrash: _data.performance?.autoDeleteImageTrash === true
    readonly property bool autoDeleteVideoTrash: _data.performance?.autoDeleteVideoTrash === true

    readonly property int maxThumbJobs: _data.performance?.maxThumbJobs ?? 16

    readonly property string colorSource: _data.colorSource ?? "ollama"

    readonly property string matugenConfig: cacheDir + "/matugen-config.toml"
    readonly property string defaultMatugenConfig: _resolve(_data.defaultMatugenConfig ?? "~/.config/matugen/config.toml")
    readonly property string externalMatugenCommand: _data.externalMatugenCommand ?? "matugen -c %config% image %path%"
    readonly property string matugenScheme: (_data.matugen && _data.matugen.schemeType) ? _data.matugen.schemeType : "scheme-fidelity"
    readonly property string matugenMode: (_data.matugen && _data.matugen.mode) ? _data.matugen.mode : "dark"

    readonly property var integrations: _data.integrations ?? []
    onIntegrationsChanged: _generateMatugenConfig()

    property var _matugenConfigWriter: FileView { id: matugenConfigWriter }
    function _generateMatugenConfig() {
        if (!matugenEnabled) return
        var ints = integrations
        if (!ints || ints.length === 0) return
        var tDir = templateDir
        var lines = ["[config]", "reload_apps = false", ""]
        for (var i = 0; i < ints.length; i++) {
            var integ = ints[i]
            if (!integ.template) continue
            var inputPath = integ.template.indexOf("/") >= 0
                ? _resolve(integ.template)
                : tDir + "/" + integ.template
            var outputPath = integ.output
                ? (integ.output.indexOf("/") >= 0
                    ? _resolve(integ.output)
                    : cacheDir + "/" + integ.output)
                : ""
            if (!outputPath) continue
            var safe = (integ.name || "integration_" + i).replace(/[^a-zA-Z0-9_-]/g, "_")
            lines.push("[templates." + safe + "]")
            lines.push('input_path = "' + inputPath + '"')
            lines.push('output_path = "' + outputPath + '"')
            lines.push("")
        }
        matugenConfigWriter.path = matugenConfig
        matugenConfigWriter.setText(lines.join("\n"))
        console.log("Config: generated matugen config with", ints.length, "integrations")
    }

    Component.onCompleted: console.log("Configuration Loaded")

    property var _components: _data.components ?? {}
    property var _wallpaperSelector: (typeof _components.wallpaperSelector === "object" && _components.wallpaperSelector !== null) ? _components.wallpaperSelector : {}

    readonly property var _screen: Quickshell.screens[0] ?? null
    readonly property int _screenW: _screen ? _screen.width : 1920
    readonly property int _screenH: _screen ? _screen.height : 1080
    readonly property bool _isSmallScreen: _screenW <= 1600

    readonly property int wallpaperSliceHeight: _wallpaperSelector.sliceHeight ?? (_isSmallScreen ? 360 : 520)
    readonly property int wallpaperVisibleCount: _wallpaperSelector.visibleCount ?? (_isSmallScreen ? 8 : 12)
    readonly property int wallpaperExpandedWidth: _wallpaperSelector.expandedWidth ?? (_isSmallScreen ? 600 : 924)
    readonly property int wallpaperSliceWidth: _wallpaperSelector.sliceWidth ?? (_isSmallScreen ? 90 : 135)
    readonly property int wallpaperSliceSpacing: _wallpaperSelector.sliceSpacing ?? -30
    readonly property int wallpaperSkewOffset: _wallpaperSelector.skewOffset ?? (_isSmallScreen ? 25 : 35)
    readonly property bool wallpaperSliceRoundCorners: _wallpaperSelector.roundCorners === true
    readonly property int wallpaperSliceCornerRadius: wallpaperSliceRoundCorners ? (_wallpaperSelector.cornerRadius ?? 16) : 0
    readonly property var wallpaperCustomPresets: _wallpaperSelector.customPresets ?? {}

    readonly property string displayMode: _wallpaperSelector.displayMode ?? "slices"
    readonly property int hexRadius: _wallpaperSelector.hexRadius ?? (_isSmallScreen ? 100 : 140)
    readonly property int hexRows: _wallpaperSelector.hexRows ?? 3
    readonly property int hexCols: _wallpaperSelector.hexCols ?? (_isSmallScreen ? 5 : 7)
    readonly property int hexScrollStep: _wallpaperSelector.hexScrollStep ?? 1
    readonly property bool hexArc: _wallpaperSelector.hexArc !== false
    readonly property real hexArcIntensity: _wallpaperSelector.hexArcIntensity ?? 1.2

    readonly property int gridColumns: _wallpaperSelector.gridColumns ?? (_isSmallScreen ? 4 : 6)
    readonly property int gridRows: _wallpaperSelector.gridRows ?? 3
    readonly property int gridThumbWidth: _wallpaperSelector.gridThumbWidth ?? (_isSmallScreen ? 220 : 300)
    readonly property int gridThumbHeight: _wallpaperSelector.gridThumbHeight ?? (_isSmallScreen ? 124 : 169)

    readonly property int mosaicCells: _wallpaperSelector.mosaicCells ?? (_isSmallScreen ? 30 : 48)
    readonly property int mosaicSeed: _wallpaperSelector.mosaicSeed ?? 7
    readonly property int mosaicRelaxation: _wallpaperSelector.mosaicRelaxation ?? 2
    readonly property int mosaicWidth: _wallpaperSelector.mosaicWidth ?? (_isSmallScreen ? 1100 : 1500)
    readonly property int mosaicHeight: _wallpaperSelector.mosaicHeight ?? (_isSmallScreen ? 600 : 800)

    readonly property int wallhavenColumns: _wallpaperSelector.wallhavenColumns ?? (_isSmallScreen ? 4 : 6)
    readonly property int wallhavenRows: _wallpaperSelector.wallhavenRows ?? 3
    readonly property int wallhavenThumbWidth: _wallpaperSelector.wallhavenThumbWidth ?? (_isSmallScreen ? 220 : 300)
    readonly property int wallhavenThumbHeight: _wallpaperSelector.wallhavenThumbHeight ?? (_isSmallScreen ? 124 : 169)
    readonly property string wallhavenApiKey: Quickshell.env("WALLHAVEN_API_KEY") || (_data.wallhaven?.apiKey ?? "")

    readonly property int steamColumns: _wallpaperSelector.steamColumns ?? (_isSmallScreen ? 4 : 6)
    readonly property int steamRows: _wallpaperSelector.steamRows ?? 3
    readonly property int steamThumbWidth: _wallpaperSelector.steamThumbWidth ?? (_isSmallScreen ? 220 : 300)
    readonly property int steamThumbHeight: _wallpaperSelector.steamThumbHeight ?? (_isSmallScreen ? 124 : 169)
    readonly property string steamApiKey: Quickshell.env("STEAM_API_KEY") || (_data.steam?.apiKey ?? "")
    readonly property string steamUsername: _data.steam?.username ?? ""

    readonly property var postProcessing: _data.postProcessing ?? []
    readonly property bool postProcessOnRestore: _data.postProcessOnRestore === true

    readonly property bool isKDE: {
        var desktop = (Quickshell.env("XDG_CURRENT_DESKTOP") || "").toLowerCase()
        return desktop.indexOf("kde") >= 0 || desktop.indexOf("plasma") >= 0
    }

    readonly property string kdeVideoPlugin: "luisbocanegra.smart.video.wallpaper.reborn"
}
