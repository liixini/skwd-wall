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

    SettingsCard {
        colors: root.colors
        title: "Tagging prompt"
        subtitle: "Override the built-in prompt sent to Ollama. Leave the editor blank to use the daemon's default."
        width: parent.width

        Rectangle {
            width: parent.width - 28
            anchors.horizontalCenter: parent ? parent.horizontalCenter : undefined
            height: 220 * Config.uiScale
            radius: 6
            color: root.colors ? Qt.rgba(root.colors.surfaceContainer.r, root.colors.surfaceContainer.g, root.colors.surfaceContainer.b, 0.6) : Qt.rgba(0.1, 0.12, 0.18, 0.6)
            border.width: promptEdit.activeFocus ? 1 : 0
            border.color: root.colors ? Qt.rgba(root.colors.primary.r, root.colors.primary.g, root.colors.primary.b, 0.5) : Qt.rgba(1, 1, 1, 0.3)

            Flickable {
                anchors.fill: parent
                anchors.margins: 8
                contentWidth: width
                contentHeight: promptEdit.contentHeight
                clip: true
                boundsBehavior: Flickable.StopAtBounds

                TextEdit {
                    id: promptEdit
                    width: parent.width
                    text: Config.ollamaPrompt
                    color: root.colors ? root.colors.surfaceText : "#fff"
                    font.family: Style.fontFamilyCode
                    font.pixelSize: 11 * Config.uiScale
                    wrapMode: TextEdit.Wrap
                    selectByMouse: true
                    selectByKeyboard: true
                    persistentSelection: true
                    textFormat: TextEdit.PlainText
                    selectionColor: root.colors ? Qt.rgba(root.colors.primary.r, root.colors.primary.g, root.colors.primary.b, 0.4) : Qt.rgba(0.5, 0.7, 1.0, 0.4)
                    onTextChanged: _promptSaveDebounce.restart()

                    Text {
                        anchors.fill: parent
                        text: "Leave blank to use the built-in default. Click “Load default into editor” to start from the daemon's baseline."
                        color: root.colors ? Qt.rgba(root.colors.surfaceText.r, root.colors.surfaceText.g, root.colors.surfaceText.b, 0.3) : Qt.rgba(1, 1, 1, 0.25)
                        font: promptEdit.font
                        wrapMode: Text.WordWrap
                        visible: promptEdit.text.length === 0 && !promptEdit.activeFocus
                    }
                }
            }

            Timer {
                id: _promptSaveDebounce
                interval: 500
                onTriggered: {
                    if (root.saveConfigKey) root.saveConfigKey("ollama.prompt", promptEdit.text)
                }
            }
        }

        RowAction {
            colors: root.colors
            title: "Load default into editor"
            description: "Fetch the daemon's built-in prompt and put it in the editor so you can tweak from it."
            onClicked: {
                DaemonClient.call("analysis.default_prompt", {}, function(result, err) {
                    if (err || !result || typeof result.prompt !== "string") return
                    promptEdit.text = result.prompt
                })
            }
        }

        RowAction {
            colors: root.colors
            title: "Reset to built-in"
            description: "Clear the override so the daemon uses its built-in prompt."
            onClicked: {
                promptEdit.text = ""
                if (root.saveConfigKey) root.saveConfigKey("ollama.prompt", "")
            }
        }
    }
}
