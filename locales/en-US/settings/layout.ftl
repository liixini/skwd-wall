settings-layout-style = Style
settings-layout-style-desc = The layout family, or a premade layout to start from. Your saved layouts are at the top.
settings-layout-style-say = The picker uses { $mode }. { $premade }.
settings-layout-style-unsaved = The picker uses { $mode }. { $premade } ({ $name }, unsaved).
settings-layout-premade-start = Start from a premade
settings-layout-premade-head = Premade layouts
settings-layout-premade-menu = Premade · a full layout, as unsaved changes
settings-layout-use = Use
settings-layout-premade-classic = Classic slices
settings-layout-premade-classic-desc = Twelve tall slices, the selected one wide open.
settings-layout-premade-compact = Compact geometric
settings-layout-premade-compact-desc = Small hexagon cells, two rows of nine.
settings-layout-premade-gallery = Gallery wall
settings-layout-premade-gallery-desc = A justified wall of seven columns.
settings-layout-premade-sandy = Sandy ribbons
settings-layout-premade-sandy-desc = Sandy slices that weave through ten ribbons.
settings-layout-premade-hand = Card hand fan
settings-layout-premade-hand-desc = Seven cards in a wide fan.
settings-layout-premade-depth = Depth stack
settings-layout-premade-depth-desc = Nine cards receding close together.
settings-layout-shape = Shape
settings-layout-slices-desc = Size and motion of the Slices strip.
settings-layout-slices-say = { $visible } visible items, { $width } wide when selected, skew { $skew }.
settings-layout-size = Size
settings-layout-tilt = Tilt and motion
settings-layout-corners-desc = Straight, round or chamfered slice corners, one value per corner.
settings-layout-corners-straight = Corner style { $style }: square corners.
settings-layout-corners-all = Corner style { $style }, radius { $radius } on every corner.
settings-layout-corners-each = Corner style { $style }, radius { $tl }, { $tr }, { $bl }, { $br }, top row then bottom.
settings-layout-corners-all-cut = Corner style { $style }, cut { $cut } on every corner.
settings-layout-corners-each-cut = Corner style { $style }, cut { $tl }, { $tr }, { $bl }, { $br }, top row then bottom.
settings-layout-corner-style = Corner style
settings-layout-corner-straight = Straight
settings-layout-corner-round = Round
settings-layout-corner-chamfered = Chamfered
settings-layout-corners-same = Same for all corners
settings-layout-corners-hint = Radius in px
settings-layout-corners-hint-cut = Cut in px
settings-layout-corner-all-radius = Radius for all corners
settings-layout-corner-all-cut = Cut for all corners
settings-layout-corner-tl-radius = Top-left radius
settings-layout-corner-tr-radius = Top-right radius
settings-layout-corner-br-radius = Bottom-right radius
settings-layout-corner-bl-radius = Bottom-left radius
settings-layout-corner-tl-cut = Top-left cut
settings-layout-corner-tr-cut = Top-right cut
settings-layout-corner-br-cut = Bottom-right cut
settings-layout-corner-bl-cut = Bottom-left cut
settings-layout-hand-desc = The hand of cards and how it fans out.
settings-layout-hand-say = { $count } cards, dealt with { $move }, { $spread } apart.
settings-layout-hand-count = 3 to 9
settings-layout-hand-moves = Random deal moves
settings-layout-fan = Fan
settings-layout-ribbons-desc = Cards cut into ribbons that swing open when a card flips.
settings-layout-ribbons-say = { $count } ribbons cut in { $axis }; { $style ->
        [straight] straight
        [chamfered] chamfered
       *[round] round
    } corners.
settings-layout-cuts = Cuts
settings-layout-atmosphere-desc = The mood around the hand.
settings-layout-atmosphere-say = Ghost trails { $ghosts }, blurred backdrop { $backdrop }, idle bob { $bob }.
settings-layout-sandy-desc = Sandy slices and how heavy the sand is.
settings-layout-sandy-say = Grain size { $grain } at { $scale } render scale. { $meter }
settings-layout-sandy-slices = Sandy slices
settings-layout-sand-desc = How the old wallpaper swaps for the new.
settings-layout-sand-say = { $style } swap over { $duration } ms; corner loop { $loop }.
settings-layout-swap = Swap
settings-layout-hand-off = Hand-off
settings-layout-flow = Grain flow
settings-layout-flow-desc = How the grains move in flight.
settings-layout-flow-say = { $ribbons } ribbons, twist { $twist } %, turbulence { $turbulence } %.
settings-layout-ring-desc = The ring of choices around the hero.
settings-layout-ring-say = The ring is { $size } % wide and settles after { $hold } ms.
settings-layout-hex-desc = The geometric field and its cells.
settings-layout-hex-say = { $tiles } tiles, { $curve } mode, { $rows } rows by { $columns } columns.
settings-layout-grid-cells = Grid and cells
settings-layout-wall-desc = How the wall arranges its cards.
settings-layout-wall-say = { $mode } wall, { $columns } columns of { $width } wide cards.
settings-layout-depth-desc = Depth cards and how they move.
settings-layout-depth-say = { $count } visible cards, { $spacing } apart, { $speed } per move.
settings-layout-look = Look
settings-layout-collection-desc = The stacked collection.
settings-layout-collection-say = { $count } visible cards at { $size } size, tilted { $tilt }.
settings-layout-stack = Stack
settings-layout-placement = Placement
settings-layout-placement-desc = This style's picker place; other styles keep their own. Filter bar and search panel are shared by every style.
settings-layout-placement-say = Picker offset { $across } across, { $down } down.
settings-layout-placement-fixed = { $mode } always sits in the centre.
settings-layout-this-style = { $mode } picker · this style only
settings-layout-shared = Shared by every style
settings-layout-pad-picker = Picker
settings-layout-pad-filter = Filter bar
settings-layout-pad-search = Search panel
settings-layout-step-percent = 1 % a press
settings-layout-step-pixels = 10 px, Shift 1 px
settings-layout-centre = Centre
settings-layout-centre-hint = Sandy strip height
settings-layout-no-offsets = { $mode } has no picker offsets today; it always sits in the centre.
settings-layout-frame-size = Frame size
settings-layout-cards = Cards and previews
settings-layout-cards-desc = Colour and video previews, the card flip, and how filter results roll.
settings-layout-cards-say = Cards flip with { $effect }; results roll { $filter }. { $video ->
        [on] Videos preview after { $delay } ms.
       *[off] Video preview is off.
    }
settings-layout-previews = Previews
settings-layout-flip-hint = Total card-flip time
settings-layout-opening = Opening and motion
settings-layout-opening-desc = Where and how the picker opens, what applying does, and the speeds its controls move at.
settings-layout-opening-say = Opens on { $monitor } at { $scale } scale, at { $start }. Appears with { $anim } at { $pace } pace. { $closes ->
        [yes] Closes on apply.
       *[no] Stays open on apply.
    }
settings-layout-opening-still = Opens on { $monitor } at { $scale } scale, at { $start }. Appears with { $anim }. { $closes ->
        [yes] Closes on apply.
       *[no] Stays open on apply.
    }
settings-layout-opening-group = Opening and closing
settings-layout-motion-group = Motion and input
settings-layout-speeds = Fast · Standard · Slow
settings-layout-animation-none = no animation
settings-layout-animation-fade = a fade
settings-layout-animation-rise = a rise
settings-layout-animation-zoom = a zoom
settings-layout-after-applying = After applying
settings-layout-not-saved = Not in saved layouts
settings-layout-fade-hint = Opacity the entrance starts from
settings-layout-follows = Follows the pointer
settings-layout-wraps = Wraps around
settings-layout-saved = Saved layout
settings-layout-first = My layout
settings-layout-holds = A saved layout holds the style, shape, placement, cards and previews, and opening and motion settings.
settings-layout-yours = Your saved layouts
settings-layout-save-new = Save as new
settings-layout-manage = Manage { $name }
settings-layout-rename = Rename
settings-layout-duplicate = Duplicate
settings-layout-delete = Delete
settings-layout-delete-last = Delete (keep at least one)
settings-layout-delete-confirm = Confirm delete { $name }
settings-layout-keep = Keep your { $count ->
       [one] { $count } change
      *[other] { $count } changes
    }?
settings-layout-save-first = Save first
settings-layout-discard = Discard
settings-layout-cancel = Cancel
settings-layout-state-saved = saved
settings-layout-state-on = on
settings-layout-state-off = off
settings-layout-changes = { $count ->
       [one] { $count } change
      *[other] { $count } changes
    } since saved
settings-layout-save = Save
settings-layout-revert = Revert
settings-layout-copy = { $name } copy
settings-layout-next = { $name } 2
settings-layout-rejected = Could not apply that layout. Nothing changed.
