import QtQuick
import ".."
import "../.."
import "../../components"

Column {
    id: root
    property var colors
    property var saveConfigKey

    width: parent ? parent.width : 0
    spacing: 8

    readonly property var _shaderOptions: [
        { key: "random",             label: "Random" },
        { key: "bounce",             label: "Bounce" },
        { key: "chromatic-bloom",    label: "Chromatic Bloom" },
        { key: "circle-crop",        label: "Circle Crop" },
        { key: "colour-distance",    label: "Colour Distance" },
        { key: "crazy-parametric",   label: "Crazy Parametric" },
        { key: "crosswarp",          label: "Cross Warp" },
        { key: "crosshatch",         label: "Crosshatch" },
        { key: "directional",        label: "Directional" },
        { key: "directional-scaled", label: "Directional Scaled" },
        { key: "directional-wipe",   label: "Directional Wipe" },
        { key: "edge-transition",    label: "Edge Transition" },
        { key: "fadecolor",          label: "Fadecolor" },
        { key: "flyeye",             label: "Fly Eye" },
        { key: "glitch",             label: "Glitch" },
        { key: "glitch-displace",    label: "Glitch Displace" },
        { key: "heat-melt",          label: "Heat Melt" },
        { key: "ink-splash",         label: "Ink Splash" },
        { key: "inkwell-drop",       label: "Inkwell Drop" },
        { key: "iris",               label: "Iris" },
        { key: "liquid-ripple",      label: "Liquid Ripple" },
        { key: "morph",              label: "Morph" },
        { key: "mosaic-tumble",      label: "Mosaic Tumble" },
        { key: "overexposure",       label: "Overexposure" },
        { key: "parametric-glitch",  label: "Parametric Glitch" },
        { key: "perlin",             label: "Perlin" },
        { key: "pixelate",           label: "Pixelate" },
        { key: "pixelfade-wave",     label: "Pixelfade Wave" },
        { key: "plasma-flow",        label: "Plasma Flow" },
        { key: "polar-function",     label: "Polar Function" },
        { key: "polka-dots-curtain", label: "Polka Dots Curtain" },
        { key: "puzzle-right",       label: "Puzzle Right" },
        { key: "randomsquares",      label: "Randomsquares" },
        { key: "smoke",              label: "Smoke" },
        { key: "soft-warp-fade",     label: "Soft Warp Fade" },
        { key: "static-fade",        label: "Static Fade" },
        { key: "voronoi-shatter",    label: "Voronoi Shatter" },
        { key: "wave-warp",          label: "Wave Warp" },
        { key: "zoom-blur-pull",     label: "Zoom Blur Pull" }
    ]

    SettingsCard {
        colors: root.colors
        title: "Engine"
        subtitle: "Which program puts pixels on the screen for static images. Video and Wallpaper Engine items always use the built-in path."

        SettingsRow {
            colors: root.colors
            title: "Wallpaper engine"
            description: "I prefer Skwd-paper because I made it, but use Awww if that's your thing :)"
            Row {
                spacing: 4
                Repeater {
                    model: [
                        { key: "skwd-paper", label: "Skwd-paper" },
                        { key: "awww",       label: "awww" }
                    ]
                    FilterButton {
                        colors: root.colors
                        label: modelData.label
                        skew: 8 * Config.uiScale; height: 26 * Config.uiScale
                        isActive: Config.wallpaperEngine === modelData.key
                        onClicked: Config.saveKey("paper.engine", modelData.key)
                    }
                }
            }
        }

        SettingsRow {
            colors: root.colors
            title: "Fill mode"
            description: "How the wallpaper is fitted to the screen. Applies to images and videos."
            Row {
                spacing: 4
                Repeater {
                    model: [
                        { key: "fill",    label: "Fill" },
                        { key: "fit",     label: "Fit" },
                        { key: "stretch", label: "Stretch" },
                        { key: "center",  label: "Center" },
                        { key: "tile",    label: "Tile" }
                    ]
                    FilterButton {
                        colors: root.colors
                        label: modelData.label
                        skew: 8 * Config.uiScale; height: 26 * Config.uiScale
                        isActive: Config.fillMode === modelData.key
                        onClicked: Config.saveKey("display.fillMode", modelData.key)
                    }
                }
            }
        }
    }

    SettingsCard {
        visible: Config.wallpaperEngine === "skwd-paper"
        colors: root.colors
        title: "Transitions"
        subtitle: "Shader-based transitions between wallpaper applies."

        RowToggle {
            colors: root.colors
            title: "Enable transitions"
            description: "Cross-fade between wallpapers using a shader."
            checked: Config.transitionEnabled
            onToggle: function(v) { Config.saveKey("transition.enabled", v) }
        }

        RowInput {
            colors: root.colors
            title: "Duration (ms)"
            description: "Transition length in milliseconds."
            value: Config.transitionDurationMs
            min: 100; max: 10000
            onCommit: function(v) { Config.saveKey("transition.durationMs", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Random shader per transition"
            description: "Pick a different shader for every transition."
            checked: Config.transitionShader === "random"
            onToggle: function(v) {
                if (v) {
                    if (Config.transitionShader !== "random" && root.saveConfigKey)
                        root.saveConfigKey("transition.lastShader", Config.transitionShader)
                    if (root.saveConfigKey) root.saveConfigKey("transition.shader", "random")
                } else {
                    var fallback = (Config._data.transition && Config._data.transition.lastShader) || "morph"
                    if (fallback === "random") fallback = "morph"
                    if (root.saveConfigKey) root.saveConfigKey("transition.shader", fallback)
                }
            }
        }

        ShaderPicker {
            colors: root.colors
            model: root._shaderOptions.filter(function(s) { return s.key !== "random" })
            value: Config.transitionShader
            enabled: Config.transitionEnabled && Config.transitionShader !== "random"
            opacity: (Config.transitionEnabled && Config.transitionShader !== "random") ? 1.0 : 0.4
            onSelected: function(key) { if (root.saveConfigKey) root.saveConfigKey("transition.shader", key) }
        }
    }

    SettingsCard {
        visible: Config.wallpaperEngine === "awww"
        colors: root.colors
        title: "AWWW transition"

        SettingsRow {
            colors: root.colors
            title: "Type"
            Flow {
                width: 360
                spacing: 4
                Repeater {
                    model: [
                        { key: "none",   label: "None" },   { key: "simple", label: "Simple" },
                        { key: "fade",   label: "Fade" },   { key: "wipe",   label: "Wipe" },
                        { key: "wave",   label: "Wave" },   { key: "grow",   label: "Grow" },
                        { key: "center", label: "Center" }, { key: "outer",  label: "Outer" },
                        { key: "left",   label: "Left" },   { key: "right",  label: "Right" },
                        { key: "top",    label: "Top" },    { key: "bottom", label: "Bottom" },
                        { key: "any",    label: "Any" },    { key: "random", label: "Random" }
                    ]
                    FilterButton {
                        colors: root.colors
                        label: modelData.label
                        skew: 8 * Config.uiScale; height: 24 * Config.uiScale
                        isActive: Config.awwwTransitionType === modelData.key
                        onClicked: Config.saveKey("paper.awww.transitionType", modelData.key)
                    }
                }
            }
        }

        RowInput {
            colors: root.colors
            title: "Duration (ms)"
            value: Config.awwwTransitionDurationMs
            min: 100; max: 10000
            onCommit: function(v) { Config.saveKey("paper.awww.transitionDurationMs", v) }
        }

        RowInput {
            colors: root.colors
            title: "FPS"
            value: Config.awwwTransitionFps
            min: 15; max: 240
            onCommit: function(v) { Config.saveKey("paper.awww.transitionFps", v) }
        }

        RowInput {
            colors: root.colors
            title: "Step (1-255)"
            value: Config.awwwTransitionStep
            min: 1; max: 255
            onCommit: function(v) { Config.saveKey("paper.awww.transitionStep", v) }
        }

        RowInput {
            visible: Config.awwwTransitionType === "wipe" || Config.awwwTransitionType === "wave"
            colors: root.colors
            title: "Angle (deg)"
            value: Config.awwwTransitionAngle
            min: 0; max: 360
            onCommit: function(v) { Config.saveKey("paper.awww.transitionAngle", v) }
        }

        RowInput {
            visible: Config.awwwTransitionType === "wave"
            colors: root.colors
            title: "Wave width"
            value: Config.awwwTransitionWaveWidth
            min: 1; max: 500
            onCommit: function(v) { Config.saveKey("paper.awww.transitionWaveWidth", v) }
        }

        RowInput {
            visible: Config.awwwTransitionType === "wave"
            colors: root.colors
            title: "Wave height"
            value: Config.awwwTransitionWaveHeight
            min: 1; max: 500
            onCommit: function(v) { Config.saveKey("paper.awww.transitionWaveHeight", v) }
        }

        RowTextInput {
            visible: Config.awwwTransitionType === "grow" || Config.awwwTransitionType === "outer"
            colors: root.colors
            title: "Position"
            description: "e.g. center, top-left, 0.5,0.5, 200,400"
            value: Config.awwwTransitionPos
            placeholder: "center"
            onCommit: function(v) { Config.saveKey("paper.awww.transitionPos", v) }
        }

        RowToggle {
            visible: Config.awwwTransitionType === "grow" || Config.awwwTransitionType === "outer"
            colors: root.colors
            title: "Invert Y"
            description: "Flip the Y axis of the transition origin."
            checked: Config.awwwInvertY
            onToggle: function(v) { Config.saveKey("paper.awww.invertY", v) }
        }

        RowTextInput {
            visible: Config.awwwTransitionType === "fade"
            colors: root.colors
            title: "Bezier"
            description: "Cubic Bezier control points: x1,y1,x2,y2"
            value: Config.awwwTransitionBezier
            placeholder: "0.0,0.0,1.0,1.0"
            onCommit: function(v) { Config.saveKey("paper.awww.transitionBezier", v) }
        }

        SettingsRow {
            colors: root.colors
            title: "Filter"
            description: "Image resampling filter."
            Row {
                spacing: 4
                Repeater {
                    model: [
                        { key: "Nearest",    label: "Nearest" },
                        { key: "Bilinear",   label: "Bilinear" },
                        { key: "CatmullRom", label: "CatmullRom" },
                        { key: "Mitchell",   label: "Mitchell" },
                        { key: "Lanczos3",   label: "Lanczos3" }
                    ]
                    FilterButton {
                        colors: root.colors
                        label: modelData.label
                        skew: 8 * Config.uiScale; height: 26 * Config.uiScale
                        isActive: Config.awwwFilter === modelData.key
                        onClicked: Config.saveKey("paper.awww.filter", modelData.key)
                    }
                }
            }
        }
    }
}
