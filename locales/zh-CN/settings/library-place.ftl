settings-library-folders-desc = 扫描这些文件夹及其子文件夹中的图片和视频。
settings-library-folders-say = 正在扫描 { $val }{ $folders }{ $end }。{ $state ->
        [polling] 轮询每 { $every } 补上原生监视遗漏的内容。{ $unread }
        [native] 原生文件监视已启用，轮询空闲。
        [down] 原生监视失败，轮询已关闭。
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] 有一个文件夹无法读取。
       *[other] 有 { $count } 个文件夹无法读取。
    }
settings-library-watching = 监视
settings-library-poll-hint = 用于网络或 FUSE 文件夹
settings-library-interval-hint = 更改后请重启 skwd-walld
settings-library-excluded-desc = 跳过这些文件夹及其子文件夹。现有元数据会保留。
settings-library-excluded-say = 从不扫描 { $val }{ $folders }{ $end }。{ $outside }
settings-library-outside = { $count ->
        [one] 有一个不在任何壁纸文件夹内,不会跳过任何内容。
       *[other] 有 { $count } 个不在任何壁纸文件夹内,不会跳过任何内容。
    }
settings-library-excluded-hint = 排除的文件夹从不扫描,因此任何配置文件都无法显示。要保留文件夹但在公开场合隐藏,请使用 { $link }。
settings-library-profiles-desc = 决定选择器、文件夹筛选、随机壁纸和计划使用哪些文件夹。
settings-library-profiles-say = 当前配置文件 { $state ->
        [none] { $menu },显示所有文件夹。
       *[some] { $menu }:{ $summary }。
    }Ctrl+H 或栏上的隐藏开关可显示隐藏的壁纸,包括从卡片隐藏的壁纸,直到下次启动。
settings-library-active = 当前配置文件
settings-library-terminal = 从终端切换
settings-library-tree-note = 隐藏:在按 Ctrl+H 前不参与随机。排除:按 Ctrl+H 也不显示。文件夹会设置其子文件夹。
settings-library-show-tip = 在选择器和随机中
settings-library-hide-tip = 直到 Ctrl+H,从不随机
settings-library-exclude-tip = 不在此配置文件中,Ctrl+H 也跳过
settings-library-from = 来自 { $link }
settings-library-excluded-inside = 内部 { $count } 个已排除
settings-library-both-inside = 内部 { $hidden } 个隐藏,{ $excluded } 个排除
settings-library-storage = 存储和回收站
settings-library-storage-desc = 更小的 WebP 副本、恢复存储和重新着色的副本。
settings-library-storage-say = { $optimise ->
        [on] 新图片转为 WebP,质量 { $quality },上限 { $size }。
       *[off] 新图片保持原样。全部优化使用质量 { $quality },上限 { $size }。
    }{ $trash ->
        [on] 回收站中的原件保留 { $days },然后删除。
       *[off] 回收站中的原件保留到你删除为止。
    }{ $recolour ->
        [on] 新壁纸也会生成重新着色的副本。
       *[off] 新壁纸不重新着色。
    }
settings-library-retention-hint = 回收站图片保留天数
settings-library-optimise-hint = 优化库中的所有图片
settings-library-recolour = 重新着色
settings-library-thumbnails = 缩略图和修复
settings-library-thumbnails-desc = 显示不对时重建预览和颜色数据。
settings-library-thumbnails-say = 重建时最多同时生成 { $jobs } 个缩略图。
settings-library-rebuild = 重建
settings-library-start-over = 重新开始
settings-library-recompute-hint = 来自现有缩略图
settings-library-clear-hint = 下次扫描时重新生成缩略图
