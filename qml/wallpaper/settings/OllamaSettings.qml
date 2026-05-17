import QtQuick
import "../.."
import "../../components"
import "../../services"

Flow {
    id: root
    property var colors
    property var saveConfigKey
    property var refreshModels
    property var openDeleteConfirm
    property var ollamaModels: []
    property bool ollamaModelsFetching: false

    width: parent ? parent.width : 0
    spacing: 12

    onVisibleChanged: { if (visible && refreshModels) refreshModels() }

    function _modelDropdownModel() {
        var out = []
        for (var i = 0; i < ollamaModels.length; i++) {
            out.push({ mode: ollamaModels[i], label: ollamaModels[i] })
        }
        return out
    }

    SettingsCard {
        colors: root.colors
        title: "Connection"
        width: (parent.width - parent.spacing) / 2

        RowTextInput {
            colors: root.colors
            title: "URL"
            description: "Ollama server endpoint."
            value: Config.ollamaUrl
            placeholder: "http://localhost:11434"
            onCommit: function(v) {
                if (root.saveConfigKey) root.saveConfigKey("ollama.url", v)
                if (root.refreshModels) root.refreshModels()
            }
        }

        RowDropdown {
            colors: root.colors
            title: "Model"
            description: root.ollamaModelsFetching
                ? "Fetching available models..."
                : (root.ollamaModels.length === 0 ? "No models found at the configured URL." : "Model used for image tagging.")
            value: Config.ollamaModel
            model: root._modelDropdownModel()
            onSelect: function(v) { if (root.saveConfigKey) root.saveConfigKey("ollama.model", v) }
        }

        RowDropdown {
            colors: root.colors
            title: "Consolidation model"
            description: "Model used to merge synonym tags. Can be the same as the tagging model."
            value: Config.ollamaConsolidationModel
            model: root._modelDropdownModel()
            onSelect: function(v) { if (root.saveConfigKey) root.saveConfigKey("ollama.consolidationModel", v) }
        }

        RowToggle {
            colors: root.colors
            title: "Tag consolidation"
            description: "Experimental: merge synonymous tags using the consolidation model. Alpha quality."
            checked: Config.ollamaConsolidateEnabled
            onToggle: function(v) { if (root.saveConfigKey) root.saveConfigKey("ollama.consolidateEnabled", v) }
        }

        RowAction {
            colors: root.colors
            title: "Refresh model list"
            description: "Re-fetch the list of models available on the Ollama server."
            onClicked: if (root.refreshModels) root.refreshModels()
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Data"
        width: (parent.width - parent.spacing) / 2

        RowAction {
            colors: root.colors
            title: "Consolidate tags now"
            description: "Send all existing tags to Ollama to merge synonyms into canonical forms. Requires consolidation toggled on."
            onClicked: WallpaperAnalysisService.consolidate()
        }

        RowAction {
            colors: root.colors
            title: "Delete all tags"
            description: "Clear all Ollama-generated tags. The next analysis pass will re-tag everything with the current model."
            onClicked: if (root.openDeleteConfirm) root.openDeleteConfirm()
        }
    }
}
