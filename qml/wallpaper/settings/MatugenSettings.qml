import QtQuick
import "../.."
import "../../components"

Column {
    id: root
    property var colors
    property var saveConfigKey
    property var cloneIntegrations

    width: parent ? parent.width : 0
    spacing: 8

    SettingsCard {
        colors: root.colors
        title: "External Matugen"
        subtitle: "Run Matugen alongside Skwd-wall's internal configuration."

        RowTextInput {
            colors: root.colors
            title: "Config path"
            description: "Path to an external matugen config file (e.g. from your existing setup)."
            value: Config.defaultMatugenConfig
            placeholder: "/path/to/matugen.config.toml"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("defaultMatugenConfig", v) }
        }

        RowTextInput {
            colors: root.colors
            title: "Command"
            description: "Shell command to invoke. %config% = config path, %path% = wallpaper. Matugen v4: add --source-color-index 0."
            value: Config.externalMatugenCommand
            placeholder: "matugen --config %config% image %path%"
            onCommit: function(v) { if (root.saveConfigKey) root.saveConfigKey("externalMatugenCommand", v) }
        }
    }

    SettingsCard {
        colors: root.colors
        title: "Integrations"
        subtitle: "Each entry generates themed output from a template and optionally runs a reload command."

        Repeater {
            id: integRepeater
            model: Config.integrations

            IntegrationEditorRow {
                colors: root.colors
                entryName: modelData.name || ""
                template: modelData.template || ""
                output: modelData.output || ""
                reload: modelData.reload || ""

                onSaveField: function(field, value) {
                    if (!root.cloneIntegrations || !root.saveConfigKey) return
                    var a = root.cloneIntegrations()
                    var key = field === "name" ? "name" : field
                    if (field === "reload") {
                        if (value === "" || value === undefined || value === null) delete a[index].reload
                        else a[index].reload = value
                    } else {
                        a[index][key] = value
                    }
                    root.saveConfigKey("integrations", a)
                }

                onRemoveRequested: {
                    if (!root.cloneIntegrations || !root.saveConfigKey) return
                    var a = root.cloneIntegrations()
                    a.splice(index, 1)
                    root.saveConfigKey("integrations", a)
                }
            }
        }

        RowAction {
            colors: root.colors
            title: "Add integration"
            description: "Append a new empty integration row."
            onClicked: {
                if (!root.cloneIntegrations || !root.saveConfigKey) return
                var a = root.cloneIntegrations(); a.push({ name: "", template: "", output: "" })
                root.saveConfigKey("integrations", a)
            }
        }
    }
}
