settings-automation-rotation-desc = 每当间隔到期，就从池中应用一张随机壁纸。
settings-automation-rotation-say = 每隔 { $interval }，从{ $val }{ $pool }{ $end }中随机挑选。
settings-automation-rotation-off = 已关闭。只有你自己选择时壁纸才会更换。
settings-automation-pool = { $count ->
        [none] 空池
        [one] { $first }
        [two] { $first }和{ $second }
       *[three] { $first }、{ $second }和{ $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] 收藏的{ $pool }
       *[all] { $pool }
    }
settings-automation-pool-images = 图片
settings-automation-pool-videos = 视频
settings-automation-pool-engine = Wallpaper Engine 项目
settings-automation-pool-heading = 池中包含
settings-automation-timing = 时间
settings-automation-interval-hint = 最少 10 秒
settings-automation-schedule-desc = 规则根据时间、星期、天气、电源和显示器选择壁纸和主题。第一条匹配的规则生效。
settings-automation-schedule-say = { $start ->
        [applies] 规则已开启；第一条匹配的规则{ $val }在启动时应用{ $end }。你自己选择的壁纸仍然优先。
       *[waits] 规则已开启；第一条匹配的规则{ $val }等待下一次变化{ $end }。你自己选择的壁纸仍然优先。
    }
settings-automation-schedule-off = 已关闭。在你开启之前，规则会一直保留。
settings-automation-start-hint = 服务启动时第一条匹配的规则
settings-automation-editor-hint = 用条件块构建规则
settings-automation-location = 位置和天气
settings-automation-location-desc = 用于日出日落时间和天气状况。
settings-automation-location-say = { $city }的天气，{ $lat }、{ $lon }处的日照时间。随天气更换：{ $mode }。
settings-automation-no-city = 未设城市
settings-automation-no-latitude = 未设纬度
settings-automation-no-longitude = 未设经度
settings-automation-location-hint = 城市留空则使用坐标。纬度以北为正，经度以东为正。
settings-automation-startup = 启动和通知
settings-automation-startup-desc = 服务启动时和壁纸更换时会发生什么。
settings-automation-startup-say = { $restore ->
        [restores] 启动时 Skwd { $val }重新应用上一张壁纸{ $end }。
       *[leaves] 启动时 Skwd { $val }保持桌面原样{ $end }。
    }{ $notify ->
        [notifies] 更换壁纸时{ $val }发送通知{ $end }。
       *[silent] 更换壁纸时{ $val }保持安静{ $end }。
    }
settings-automation-commands = 壁纸应用之后
settings-automation-commands-desc = 每次更换后运行你自己的命令，例如驱动另一个壁纸工具。
settings-automation-command-count = { $count ->
       *[other] 每次更换后运行{ $val }{ $count } 条命令{ $end }。
    }
settings-automation-commands-say = { $listed ->
        [none] 每次更换后{ $val }不运行任何命令{ $end }。
       *[some] { $commands }
    }{ $apply ->
        [commands] { $rerun ->
            [also] Skwd { $val }把应用交给你的命令{ $end }，命令{ $val }也会在启动恢复时运行{ $end }。
           *[skip] Skwd { $val }把应用交给你的命令{ $end }，命令{ $val }会跳过启动恢复{ $end }。
        }
       *[itself] { $rerun ->
            [also] Skwd { $val }自行应用壁纸{ $end }，命令{ $val }也会在启动恢复时运行{ $end }。
           *[skip] Skwd { $val }自行应用壁纸{ $end }，命令{ $val }会跳过启动恢复{ $end }。
        }
    }
settings-automation-pick-only-hint = 由你的命令应用
settings-automation-commands-heading = 命令
settings-automation-placeholders = 在命令中使用 %path%、%thumb%、%type% 和 %name%。
settings-automation-minutes = { $count } 分钟
settings-automation-mode-off = 关闭
settings-automation-mode-outdoors = 跟随户外天气
settings-automation-mode-playlists = 从播放列表中选择
settings-automation-mode-semantic = 在库中搜索
settings-automation-condition-clear = 晴朗
settings-automation-condition-cloudy = 多云
settings-automation-condition-rainy = 下雨
settings-automation-condition-snowy = 下雪
settings-automation-condition-stormy = 暴风雨
settings-automation-condition-foggy = 有雾
settings-automation-condition-windy = 大风
settings-automation-display-another = 换一张
settings-automation-display-exclude = 排除
settings-automation-display-keep = 保留
settings-automation-playlists-heading = 按天气的播放列表
settings-automation-hours = { $count } 小时
