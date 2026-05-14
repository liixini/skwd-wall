pragma Singleton
import QtQuick

QtObject {
    id: style

    readonly property string fontFamily: "Roboto Condensed"
    readonly property string fontFamilyHeading: "Roboto"
    readonly property string fontFamilyMono: "monospace"
    readonly property string fontFamilyCode: "Roboto Mono"
    readonly property string fontFamilyIcons: "Material Design Icons"
    readonly property string fontFamilyNerdIcons: "Symbols Nerd Font"
    readonly property int fontTiny: 9
    readonly property int fontCaption: 11
    readonly property int fontBody: 12
    readonly property int fontBodyLarge: 13
    readonly property int fontSubtitle: 14
    readonly property int fontTitle: 16
    readonly property int fontTitleLarge: 18
    readonly property int fontHeadline: 24
    readonly property int fontDisplay: 32
    readonly property int fontDisplayLarge: 36
    readonly property int fontClock: 200
    readonly property int fontClockDate: 120
    readonly property int radiusTiny: 2
    readonly property int radiusSmall: 4
    readonly property int radiusMedium: 8
    readonly property int radiusLarge: 12
    readonly property int radiusXLarge: 16
    readonly property int radiusRound: 20
    readonly property int radiusCircle: 40
    readonly property int spacingTiny: 2
    readonly property int spacingSmall: 4
    readonly property int spacingMedium: 8
    readonly property int spacingLarge: 12
    readonly property int spacingXLarge: 16
    readonly property int spacingXXLarge: 20
    readonly property int animVeryFast: 100
    readonly property int animFast: 150
    readonly property int animNormal: 200
    readonly property int animEnter: 250
    readonly property int animMedium: 300
    readonly property int animExpand: 350
    readonly property int animSlow: 400
    readonly property int animSpin: 1000

    readonly property int tooltipDelay: 500

    readonly property color fallbackAccent: "#4fc3f7"
    readonly property int borderThin: 1
    readonly property int borderMedium: 2
    readonly property int borderThick: 3
    readonly property real opacityDim: 0.35
    readonly property real opacityMuted: 0.5
    readonly property real opacitySubtle: 0.6
}
