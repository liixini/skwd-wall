settings-system-power-desc = The picker and wallpapers have separate battery limits.
settings-system-power-say = On battery { $cap ->
        [none] the picker has no extra cap ({ $fps })
       *[cap] the picker caps at { $fps }
    }{ $idle ->
        [off] , only the normal Idle pause applies ({ $seconds })
       *[on] , video stops after { $seconds } without input
    }, and { $detail ->
        [full] wallpapers keep full detail
       *[performance] wallpapers switch to performance mode
    }.
settings-system-power-off = Off. On battery, the picker and wallpapers run with the same limits as on mains power.
settings-system-power-source = Current power source: { $source }
settings-system-always = Always
settings-system-on-battery = On battery · while the saver is on
settings-system-max-fps-hint = Refresh rate, or 60 for a cooler laptop
settings-system-idle-hint = 0 disables
settings-system-battery-fps-hint = Lower of this and Max FPS · 0 disables
settings-system-battery-idle-hint = Shorter one wins · 0 disables
settings-system-battery-wallpaper-hint = Direct video, 30 fps scenes
settings-system-gpu-desc = Reopen the picker after changing it.
settings-system-gpu-say = { $mode ->
        [low] The picker uses { $gpu }: it asks for an integrated GPU.
        [high] The picker uses { $gpu }: it asks for a discrete GPU.
        [none] The picker uses { $gpu }: the driver decides.
        [auto] The picker uses { $gpu }: it prefers a low-power GPU while the battery saver is active.
       *[custom] The picker uses { $gpu }, which it does not recognise, so it behaves like Auto.
    }
settings-system-language-desc = Changes apply immediately and are saved.
settings-system-language-say = { $follow ->
        [desktop] The interface is in { $language }, which follows your desktop language. Wallpaper names and tags stay as they are.
       *[chosen] The interface is in { $language }. Wallpaper names and tags stay as they are.
    }
settings-system-bug-title = Bug report
settings-system-bug-desc = One file to attach to an issue.
settings-system-bug-say = Bundle version, environment, and recent logs into one file.
