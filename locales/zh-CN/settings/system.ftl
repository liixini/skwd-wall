settings-system-power-desc = 选择器和壁纸有各自的电池限制。
settings-system-power-say = 使用电池时，{ $cap ->
        [none] 选择器没有额外上限（{ $fps }）
       *[cap] 选择器上限为 { $fps }
    }{ $idle ->
        [off] ，只适用常规的空闲暂停（{ $seconds }）
       *[on] ，无输入 { $seconds } 后视频停止
    }，并且{ $detail ->
        [full] 壁纸保持完整细节
       *[performance] 壁纸切换到性能模式
    }。
settings-system-power-off = 关闭。使用电池时，选择器和壁纸采用与接通电源时相同的限制。
settings-system-power-source = 当前电源：{ $source }
settings-system-always = 始终
settings-system-on-battery = 使用电池 · 省电开启时
settings-system-max-fps-hint = 显示器刷新率，或设为 60 让笔记本更凉爽
settings-system-idle-hint = 0 表示关闭
settings-system-battery-fps-hint = 取此值与最大帧率中较低者 · 0 表示关闭
settings-system-battery-idle-hint = 较短者生效 · 0 表示关闭
settings-system-battery-wallpaper-hint = 直接播放视频，场景 30 fps
settings-system-gpu-desc = 更改后请重新打开选择器。
settings-system-gpu-say = { $mode ->
        [low] 选择器使用 { $gpu }：它会请求集成 GPU。
        [high] 选择器使用 { $gpu }：它会请求独立 GPU。
        [none] 选择器使用 { $gpu }：由驱动决定。
        [auto] 选择器使用 { $gpu }：省电模式开启时，它优先选择低功耗 GPU。
       *[custom] 选择器使用 { $gpu }，但无法识别该值，因此按“自动”处理。
    }
settings-system-language-desc = 更改会立即生效并保存。
settings-system-language-say = { $follow ->
        [desktop] 界面语言为 { $language }，跟随桌面语言。壁纸名称和标签保持不变。
       *[chosen] 界面语言为 { $language }。壁纸名称和标签保持不变。
    }
settings-system-bug-title = 错误报告
settings-system-bug-desc = 一个可附加到问题中的文件。
settings-system-bug-say = 将版本、环境和最近的日志打包成一个文件。
