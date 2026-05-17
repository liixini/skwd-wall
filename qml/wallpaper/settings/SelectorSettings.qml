import QtQuick
import ".."
import "../.."
import "../../components"

Flow {
    id: root
    property var colors
    property var saveField
    property var saveConfigKey
    property var showWarning
    property var applyPreset
    property var saveCustomPreset
    property var loadCustomPreset

    width: parent ? parent.width : 0
    spacing: 8

    SettingsCard {
        colors: root.colors
        title: "Layout"
        width: parent.width

        SettingsRow {
            colors: root.colors
            title: "Display mode"
            description: "Slices, Hex grid, Wall grid, or Mosaic."
            Row {
                spacing: -4
                Repeater {
                    model: [
                        { key: "slices",  label: "Slices" },
                        { key: "hex",     label: "Hex" },
                        { key: "wall",    label: "Wall" },
                        { key: "mosaic",  label: "Mosaic" }
                    ]
                    FilterButton {
                        colors: root.colors
                        label: modelData.label
                        skew: 8 * Config.uiScale; height: 26 * Config.uiScale
                        isActive: Config.displayMode === modelData.key
                        onClicked: {
                            if (modelData.key === "mosaic" && Config.displayMode !== "mosaic" && root.showWarning)
                                root.showWarning("MOSAIC IS EXPERIMENTAL", "Not all features work yet. Please do not expect everything to function correctly.")
                            if (root.saveField) root.saveField("displayMode", modelData.key)
                        }
                    }
                }
            }
        }

        SettingsRow {
            visible: Config.displayMode === "slices"
            colors: root.colors
            title: "Size preset"
            description: "Pick a quick slice size."
            Row {
                spacing: -4
                Repeater {
                    model: [
                        { label: "XS", expanded: 360,  sliceH: 200, sliceW: 52,  visible: 20, gap: -30, skew: 16 },
                        { label: "S",  expanded: 480,  sliceH: 270, sliceW: 68,  visible: 18, gap: -30, skew: 20 },
                        { label: "M",  expanded: 768,  sliceH: 432, sliceW: 108, visible: 14, gap: -30, skew: 28 },
                        { label: "L",  expanded: 924,  sliceH: 520, sliceW: 135, visible: 12, gap: -30, skew: 35 },
                        { label: "XL", expanded: 1280, sliceH: 720, sliceW: 180, visible: 9,  gap: -30, skew: 45 }
                    ]
                    FilterButton {
                        colors: root.colors
                        label: modelData.label
                        skew: 8 * Config.uiScale; height: 26 * Config.uiScale
                        isActive: Config.wallpaperExpandedWidth === modelData.expanded && Config.wallpaperSliceHeight === modelData.sliceH
                        onClicked: if (root.applyPreset) root.applyPreset(modelData.expanded, modelData.sliceH, modelData.sliceW, modelData.visible, modelData.gap, modelData.skew)
                        tooltip: modelData.expanded + "×" + modelData.sliceH + " (16:9)"
                    }
                }
            }
        }

        SettingsRow {
            colors: root.colors
            title: "Custom presets"
            description: "Click to apply, right-click an empty slot to save the current geometry."
            Row {
                spacing: -4
                Repeater {
                    model: ["C1", "C2", "C3", "C4"]
                    FilterButton {
                        property string presetKey: modelData + "_" + Config.displayMode
                        property var presetData: Config.wallpaperCustomPresets[presetKey] || null
                        property bool isEmpty: !presetData
                        colors: root.colors
                        label: modelData
                        skew: 8 * Config.uiScale; height: 26 * Config.uiScale
                        isActive: {
                            if (isEmpty) return false
                            if (Config.displayMode === "slices") return Config.wallpaperExpandedWidth === presetData.expandedWidth && Config.wallpaperSliceHeight === presetData.sliceHeight
                            if (Config.displayMode === "hex")    return Config.hexRadius === presetData.hexRadius && Config.hexRows === presetData.hexRows && Config.hexCols === presetData.hexCols
                            if (Config.displayMode === "wall")   return Config.gridColumns === presetData.gridColumns && Config.gridRows === presetData.gridRows
                            return false
                        }
                        activeOpacity: isEmpty ? 0.35 : 1.0
                        tooltip: {
                            if (isEmpty) return "Click to save current"
                            if (Config.displayMode === "slices") return presetData.expandedWidth + "×" + presetData.sliceHeight + " - Right-click to overwrite"
                            if (Config.displayMode === "hex")    return "r" + presetData.hexRadius + " " + presetData.hexRows + "×" + presetData.hexCols + " - Right-click to overwrite"
                            if (Config.displayMode === "wall")   return presetData.gridColumns + "×" + presetData.gridRows + " " + presetData.gridThumbWidth + "×" + presetData.gridThumbHeight + " - Right-click to overwrite"
                            return ""
                        }
                        onClicked: {
                            if (isEmpty) { if (root.saveCustomPreset) root.saveCustomPreset(modelData) }
                            else { if (root.loadCustomPreset) root.loadCustomPreset(modelData) }
                        }
                        MouseArea {
                            anchors.fill: parent; acceptedButtons: Qt.RightButton
                            cursorShape: Qt.PointingHandCursor
                            onClicked: if (root.saveCustomPreset) root.saveCustomPreset(modelData)
                        }
                    }
                }
            }
        }
    }

    SettingsCard {
        colors: root.colors
        title: Config.displayMode === "hex" ? "Hex grid" : (Config.displayMode === "wall" ? "Wall" : (Config.displayMode === "mosaic" ? "Mosaic" : "Slice size"))
        width: (parent.width - parent.spacing) / 2

        RowInput { visible: Config.displayMode === "slices"; colors: root.colors; title: "Slice height"; value: Config.wallpaperSliceHeight; min: 200; max: 1200; onCommit: function(v) { if (root.saveField) root.saveField("sliceHeight", v) } }
        RowInput { visible: Config.displayMode === "slices"; colors: root.colors; title: "Visible items"; value: Config.wallpaperVisibleCount; min: 3; max: 30; onCommit: function(v) { if (root.saveField) root.saveField("visibleCount", v) } }
        RowInput { visible: Config.displayMode === "slices"; colors: root.colors; title: "Selected width"; value: Config.wallpaperExpandedWidth; min: 50; max: 1800; onCommit: function(v) { if (root.saveField) root.saveField("expandedWidth", v) } }
        RowInput { visible: Config.displayMode === "slices"; colors: root.colors; title: "Slice width"; value: Config.wallpaperSliceWidth; min: 50; max: 500; onCommit: function(v) { if (root.saveField) root.saveField("sliceWidth", v) } }
        RowInput { visible: Config.displayMode === "slices"; colors: root.colors; title: "Gap"; value: Config.wallpaperSliceSpacing; min: -500; max: 500; onCommit: function(v) { if (root.saveField) root.saveField("sliceSpacing", v) } }
        RowInput { visible: Config.displayMode === "slices"; colors: root.colors; title: "Skew"; value: Config.wallpaperSkewOffset; min: -500; max: 500; onCommit: function(v) { if (root.saveField) root.saveField("skewOffset", v) } }

        RowInput { visible: Config.displayMode === "hex"; colors: root.colors; title: "Radius"; value: Config.hexRadius; min: 60; max: 300; onCommit: function(v) { if (root.saveField) root.saveField("hexRadius", v) } }
        RowInput { visible: Config.displayMode === "hex"; colors: root.colors; title: "Rows"; value: Config.hexRows; min: 1; max: 8; onCommit: function(v) { if (root.saveField) root.saveField("hexRows", v) } }
        RowInput { visible: Config.displayMode === "hex"; colors: root.colors; title: "Columns"; value: Config.hexCols; min: 3; max: 20; onCommit: function(v) { if (root.saveField) root.saveField("hexCols", v) } }
        RowInput { visible: Config.displayMode === "hex"; colors: root.colors; title: "Scroll step"; value: Config.hexScrollStep; min: 1; max: 10; onCommit: function(v) { if (root.saveField) root.saveField("hexScrollStep", v) } }
        RowToggle { visible: Config.displayMode === "hex"; colors: root.colors; title: "Arc layout"; checked: Config.hexArc; onToggle: function(v) { if (root.saveField) root.saveField("hexArc", v) } }
        RowInput { visible: Config.displayMode === "hex" && Config.hexArc; colors: root.colors; title: "Arc intensity (×10)"; value: Math.round(Config.hexArcIntensity * 10); min: 1; max: 30; onCommit: function(v) { if (root.saveField) root.saveField("hexArcIntensity", v / 10) } }

        RowInput { visible: Config.displayMode === "wall"; colors: root.colors; title: "Columns"; value: Config.gridColumns; min: 2; max: 12; onCommit: function(v) { if (root.saveField) root.saveField("gridColumns", v) } }
        RowInput { visible: Config.displayMode === "wall"; colors: root.colors; title: "Rows"; value: Config.gridRows; min: 1; max: 8; onCommit: function(v) { if (root.saveField) root.saveField("gridRows", v) } }
        RowInput { visible: Config.displayMode === "wall"; colors: root.colors; title: "Thumb width"; value: Config.gridThumbWidth; min: 100; max: 600; onCommit: function(v) { if (root.saveField) root.saveField("gridThumbWidth", v) } }
        RowInput { visible: Config.displayMode === "wall"; colors: root.colors; title: "Thumb height"; value: Config.gridThumbHeight; min: 50; max: 400; onCommit: function(v) { if (root.saveField) root.saveField("gridThumbHeight", v) } }

        RowInput { visible: Config.displayMode === "mosaic"; colors: root.colors; title: "Cells"; value: Config.mosaicCells; min: 4; max: 200; onCommit: function(v) { if (root.saveField) root.saveField("mosaicCells", v) } }
        RowInput { visible: Config.displayMode === "mosaic"; colors: root.colors; title: "Seed"; value: Config.mosaicSeed; min: 1; max: 99999; onCommit: function(v) { if (root.saveField) root.saveField("mosaicSeed", v) } }
        RowInput { visible: Config.displayMode === "mosaic"; colors: root.colors; title: "Relax iterations"; value: Config.mosaicRelaxation; min: 0; max: 8; onCommit: function(v) { if (root.saveField) root.saveField("mosaicRelaxation", v) } }
        RowInput { visible: Config.displayMode === "mosaic"; colors: root.colors; title: "Width"; value: Config.mosaicWidth; min: 400; max: 3000; onCommit: function(v) { if (root.saveField) root.saveField("mosaicWidth", v) } }
        RowInput { visible: Config.displayMode === "mosaic"; colors: root.colors; title: "Height"; value: Config.mosaicHeight; min: 200; max: 2000; onCommit: function(v) { if (root.saveField) root.saveField("mosaicHeight", v) } }
    }

    SettingsCard {
        visible: Config.displayMode === "slices"
        colors: root.colors
        title: "Corners"
        width: (parent.width - parent.spacing) / 2

        RowToggle {
            colors: root.colors
            title: "Round corners"
            description: "Apply a corner radius to slice edges."
            checked: Config.wallpaperSliceRoundCorners
            onToggle: function(v) { if (root.saveField) root.saveField("roundCorners", v) }
        }

        RowInput {
            visible: Config.wallpaperSliceRoundCorners
            colors: root.colors
            title: "Radius"
            description: "Corner radius in pixels."
            value: Config.wallpaperSliceCornerRadius
            min: 0; max: 80
            onCommit: function(v) { if (root.saveField) root.saveField("cornerRadius", v) }
        }
    }

}
