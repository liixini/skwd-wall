import QtQuick
import "../.."
import "../../components"

Flow {
    id: root
    property var colors
    property var saveConfigKey

    width: parent ? parent.width : 0
    spacing: 12

    SettingsCard {
        colors: root.colors
        title: "Rendering"
        subtitle: "Defaults applied when linux-wallpaperengine renders a scene/web Workshop wallpaper."
        width: parent.width

        RowInput {
            colors: root.colors
            title: "FPS cap"
            description: "Maximum frames per second. Lower values reduce CPU/GPU load."
            value: Config.weRenderFps
            min: 1; max: 240
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("weRender.fps", v) }
        }

        RowDropdown {
            colors: root.colors
            title: "Default scaling"
            description: "How a scene fits each monitor."
            value: Config.weRenderScaling
            model: [
                { mode: "default", label: "Default" },
                { mode: "fill",    label: "Fill" },
                { mode: "fit",     label: "Fit" },
                { mode: "stretch", label: "Stretch" }
            ]
            onSelect: function(v) { if (root.saveConfigKey) root.saveConfigKey("weRender.scaling", v) }
        }

        RowDropdown {
            colors: root.colors
            title: "Default clamp"
            description: "How textures wrap at edges."
            value: Config.weRenderClamp
            model: [
                { mode: "border", label: "Border" },
                { mode: "repeat", label: "Repeat" },
                { mode: "mirror", label: "Mirror" }
            ]
            onSelect: function(v) { if (root.saveConfigKey) root.saveConfigKey("weRender.clamp", v) }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Pause behaviour"
        width: parent.width

        RowToggle {
            colors: root.colors
            title: "Don't pause on fullscreen"
            description: "Keep the wallpaper running even when another window is fullscreen."
            checked: Config.weRenderNoFullscreenPause
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("weRender.noFullscreenPause", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Only pause when fullscreen is active"
            description: "Wayland only. Pause only when a fullscreen window is the active one (not just present). Ignored if 'Don't pause on fullscreen' is on."
            checked: Config.weRenderFullscreenPauseOnlyActive
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("weRender.fullscreenPauseOnlyActive", v) }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Audio"
        width: parent.width

        RowToggle {
            colors: root.colors
            title: "Disable auto-mute"
            description: "Don't auto-mute the wallpaper when another app plays audio."
            checked: Config.weRenderNoautomute
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("weRender.noautomute", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Disable audio processing"
            description: "Skip processing audio for audio-reactive wallpapers."
            checked: Config.weRenderNoAudioProcessing
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("weRender.noAudioProcessing", v) }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Effects"
        width: parent.width

        RowToggle {
            colors: root.colors
            title: "Disable particles"
            description: "Skip particle effects on scenes that use them."
            checked: Config.weRenderDisableParticles
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("weRender.disableParticles", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Disable mouse interaction"
            description: "Scenes can't react to mouse position when on."
            checked: Config.weRenderDisableMouse
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("weRender.disableMouse", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Disable parallax"
            description: "Skip parallax depth effect on supporting scenes."
            checked: Config.weRenderDisableParallax
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("weRender.disableParallax", v) }
        }
    }
}
