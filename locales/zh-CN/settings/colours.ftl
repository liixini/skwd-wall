settings-colours-source = 颜色来源
settings-colours-source-desc = 跟随每张壁纸、保持一个调色板，或不改变颜色。
settings-colours-source-follow = 跟随每张壁纸
settings-colours-source-fixed = 来自一个固定调色板
settings-colours-source-off = 保持不变
settings-colours-source-say = 颜色{ $policy }。
settings-colours-source-made = 颜色{ $policy }，由 { $authority } 生成。
settings-colours-end4 = end-4
settings-colours-control-menu = 颜色控制 · 检测到的外壳
settings-colours-generator-menu = 生成器 · 仅已安装
settings-colours-theme-menu = 主题 · 预设、你的、自定义种子
settings-colours-all-presets = 全部 { $count } 个预设…
settings-colours-generator = 生成器
settings-colours-generator-desc = 把壁纸转换为调色板的程序。
settings-colours-generator-say = { $engine } 生成调色板。
settings-colours-generator-seeded = { $engine } 生成调色板，种子来自槽位 { $index }，对比度 { $contrast }。
settings-colours-generator-shell = { $shell } 从壁纸生成调色板。
settings-colours-palette = 调色板
settings-colours-palette-desc = 所有壁纸使用同一调色板。
settings-colours-palette-say = 所有壁纸使用 { $theme }。
settings-colours-idle = 颜色保持不变时不使用。
settings-colours-index-hint = 用作种子颜色的调色板槽位
settings-colours-contrast-hint = -1 到 1
settings-colours-wallust-hint = 强制 wallust 调色板，如 dark16。留空按模式决定
settings-colours-seeds-hint = 供自定义种子使用
settings-colours-config-hint = 外部 Matugen
settings-colours-command-hint = %config% %path% %scheme% %mode% %index%
settings-colours-auto = 自动
settings-colours-look = 外观
settings-colours-look-desc = 颜色扩散程度、质感以及明暗。
settings-colours-look-say = { $scheme } 风格，{ $finish } 质感，变体 { $variant }。
settings-colours-look-imported = { $shell } 的调色板，导入为 { $variant } 变体。
settings-colours-look-shell = { $shell } 自行决定外观。
settings-colours-look-fixed = 固定调色板自行决定外观。
settings-colours-no-designer = 主题设计器可在跟随壁纸或固定调色板时使用。
settings-colours-designer-hint = 手动调整或创建调色板
settings-colours-go = 颜色去向
settings-colours-go-desc = 调色板变化时跟随的应用主题。
settings-colours-find-hint = 安装应用后重新检查
settings-colours-matugen-hint = Matugen 输出开关
settings-colours-reload-hint = 文件写入后运行
settings-colours-output-heading = 输出 { $count }
settings-colours-shells = 外壳
settings-colours-shells-desc = Noctalia 和 Dank Material Shell 可以使用 Skwd 的颜色。
settings-colours-dms-hint = DMS 必须使用 Auto（壁纸）
settings-colours-mode-hint = 随每个配色方案应用
settings-colours-bin-hint = 如果不在 PATH 中
settings-colours-lock = 锁屏
settings-colours-lock-desc = 显示壁纸的锁屏。
settings-colours-lock-follow = 最后应用的壁纸
settings-colours-lock-still = 一张静态图片
settings-colours-sddm-hint = 需要管理员批准
settings-colours-go-say = { $val }{ $all } 个应用中有 { $on } 个{ $end }{ $followers ->
        [one] 跟随
       *[some] 跟随
    }你的颜色{ $changed ->
        [one] ；{ $val }{ $name } 的配置在 Skwd 之外被更改{ $end }。
        [some] ；{ $val }{ $count } 个应用在 Skwd 之外被更改{ $end }。
       *[none] 。
    }
settings-colours-outputs-say = { $state ->
        [none] 还没有集成渲染模板。
        [single] { $val }1 个集成{ $end }在每次颜色变化后把模板渲染到文件。
       *[many] { $val }{ $count } 个集成{ $end }在每次颜色变化后把模板渲染到文件。
    }
settings-colours-shells-say = Noctalia { $val }{ $noctalia ->
        [follows] 跟随 Skwd 颜色
       *[keeps] 保留自己的颜色
    }{ $end }；DMS { $val }{ $dms ->
        [follows] 跟随 Skwd 颜色
       *[keeps] 保留自己的颜色
    }{ $end }{ $caelestia ->
        [yes] ；Caelestia 壁纸同步。
       *[no] 。
    }
settings-colours-lock-say = { $state ->
        [none] 尚未设置锁屏。新的锁屏使用{ $mode }。
        [one] { $names } 使用{ $mode }。
       *[some] { $names } 使用{ $mode }。
    }
settings-colours-lock-say-static = { $state ->
        [none] 尚未设置锁屏。新的锁屏使用{ $mode }：{ $image }。
        [one] { $names } 使用{ $mode }：{ $image }。
       *[some] { $names } 使用{ $mode }：{ $image }。
    }
settings-menu-more = 更多
