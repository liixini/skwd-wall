settings-wallpapers-displays-desc = 每个显示器显示的内容及其放置方式。
settings-wallpapers-play = 壁纸如何播放
settings-wallpapers-play-desc = 引擎、绘制图层和场景。
settings-wallpapers-transitions-desc = 壁纸如何切换到下一张。
settings-wallpapers-awww-desc = awww 用于图片的过渡。
settings-wallpapers-drawing = 绘制
settings-wallpapers-drawing-desc = 已准备的壁纸会自己画出来。
settings-wallpapers-sound = 声音
settings-wallpapers-sound-desc = 音量，以及壁纸音频何时让路。
settings-wallpapers-pause-desc = 满足规则时暂停动态壁纸。
settings-wallpapers-plasma = Plasma 锁屏
settings-wallpapers-plasma-desc = Plasma 锁屏时显示哪张壁纸。
settings-wallpapers-niri = Niri 概览
settings-wallpapers-niri-desc = 在 niri 上显示。概览后方的背景。
settings-wallpapers-display-more = 更多
settings-wallpapers-lock = 锁定 { $name }
settings-wallpapers-lock-hint = 仅通过多选器更新
settings-wallpapers-layer-bottom = 底部
settings-wallpapers-gpu-hint = 回退到自动
settings-wallpapers-timeout-hint = 用于超大图片
settings-wallpapers-animated = 动态壁纸
settings-wallpapers-scenes = 场景
settings-wallpapers-scenes-hint = FPS 上限 · 缩放
settings-wallpapers-particles-hint = 仅场景
settings-wallpapers-preview-hint = 帧率
settings-wallpapers-quality-short = 沙粒至少需要 { $ms } ms。
settings-wallpapers-quality-hint = Full 不限帧率，Low 限 30 fps
settings-wallpapers-sharpen-hint = 画质降低分辨率时
settings-wallpapers-monitors = 显示器
settings-wallpapers-primary-hint = 仅为主显示器播放动画
settings-wallpapers-primary-blank = 第一个可用
settings-wallpapers-primary-only-hint = 其余使用所有显示器
settings-wallpapers-wave = 波浪
settings-wallpapers-wave-hint = 宽 × 高
settings-wallpapers-position-hint = center、top-left 或 0.5,0.5
settings-wallpapers-bezier-hint = x1,y1,x2,y2
settings-wallpapers-bezier-blank = awww 默认
settings-wallpapers-invert-hint = 翻转原点的 Y 轴
settings-wallpapers-colours-files = 颜色和文件
settings-wallpapers-preparing = 准备
settings-wallpapers-paper-ink = 纸与墨
settings-wallpapers-prepare = 准备
settings-wallpapers-prepare-all = 所有壁纸
settings-wallpapers-scope-view = 当前视图 · { $count }
settings-wallpapers-pack-ready = 处理文件已就绪。
settings-wallpapers-pack-none = 还没有处理文件。
settings-wallpapers-pack-blank = 暂无
settings-wallpapers-drawing-blocked = 绘制不可用。请先选择处理文件。
settings-wallpapers-volume = 音量
settings-wallpapers-other-apps = 其他应用
settings-wallpapers-pause-apps = 列表中的应用运行时暂停。
settings-wallpapers-pause-none = 没有开启暂停规则。
settings-wallpapers-process-hint = 支持 * 和 ?
settings-wallpapers-process-action = 选择
settings-wallpapers-process-blank = 暂无
settings-wallpapers-resume-hint = 最后一条规则解除后
settings-wallpapers-scope-all = 所有显示器
settings-wallpapers-scope-display = 窗口所在显示器
settings-wallpapers-rule-fullscreen = 全屏窗口
settings-wallpapers-rule-maximised = 最大化窗口
settings-wallpapers-rule-focused = 聚焦窗口
settings-wallpapers-rule-full-width = 全宽列
settings-wallpapers-plasma-left = 交给 Plasma
settings-wallpapers-plasma-static = 显示静态图片
settings-wallpapers-plasma-follow = 跟随上次应用的壁纸
settings-wallpapers-plasma-off-row = 不管理时不改动 Plasma。
settings-wallpapers-image-hint = 绝对路径，或以 ~/ 开头
settings-wallpapers-image-blank = 未设置图片
settings-wallpapers-dynamic-hint = 海报帧是一张静音图片
settings-wallpapers-niri-off = 已关闭。概览保留 niri 自己的背景。
settings-wallpapers-source = 来源
settings-wallpapers-look = 外观
settings-wallpapers-source-blank = 上一张壁纸
settings-wallpapers-blur = 模糊背景
settings-wallpapers-radii = 模糊半径
settings-wallpapers-radii-hint = 图片 · 视频 · 场景
settings-wallpapers-dim-hint = 0 无，100 全黑
settings-wallpapers-transitions-off = 已关闭。每张壁纸直接切到下一张。
settings-wallpapers-transitions-off-foot = 开启过渡后可选择预览、显示器和沙粒质量。
settings-wallpapers-awww-none = 图片通过 { $type } 切换。
settings-wallpapers-awww-no-type = 无过渡
settings-wallpapers-sound-muted = 壁纸音频已静音。
settings-wallpapers-displays-say = { $state ->
        [none] 没有锁定的显示器。
        [one] { $names } 已锁定。
        [some] { $names } 已锁定。
       *[detecting] 正在检测显示器。
    }没有单独放置方式的显示器使用 { $fill }。
settings-wallpapers-play-say = 图片用 { $backend }，视频用 { $val }Vulkan{ $end }，位于 { $layer } 图层。
settings-wallpapers-transitions-say = { $family } 系列{ $kind ->
        [random] ，每次不同，时长 { $duration }。
        [preset] ，使用 { $shader }，时长 { $ms } ms。
       *[shader] ，使用 { $shader }，时长 { $duration }。
    }帧率 { $rate }。
settings-wallpapers-primary-count = { $state ->
        [none] { $all } 个都没有
        [all] 全部 { $all } 个
       *[some] { $all } 个中的 { $shown } 个
    }
settings-wallpapers-awww-say = 图片播放 { $type }，滤镜 { $filter }，时长 { $duration }。
settings-wallpapers-drawing-say = { $state ->
        [on] 已准备的壁纸在 { $duration } 内画完。
       *[off] 已关闭。设为绘制的卡片仍在 { $duration } 内画完。
    }
settings-wallpapers-sound-say = 壁纸音频音量 { $volume }{ $duck ->
        [yes] { "" }，其他应用发声时暂停。
       *[no] 。
    }
settings-wallpapers-pause-say = { $rules } 时在 { $scope } 上暂停{ $apps ->
        [yes] ，列表中的应用也会暂停。
       *[no] 。
    }
settings-wallpapers-pause-rules = { $count } 条窗口规则
settings-wallpapers-plasma-say = { $state ->
        [off] 锁屏 { $mode }。
        [static] 锁屏 { $mode }: { $image }。
       *[follow] 锁屏 { $mode }，场景为{ $scenes ->
        [live] 实时渲染。
       *[poster] 海报帧。
    }
    }
settings-wallpapers-niri-say = 显示 { $source ->
       *[current] 当前壁纸
        [own] 独立来源
        [stationary] 一张固定壁纸
    }{ $blur ->
        [yes] ，模糊
       *[no] { "" }
    }，调暗 { $dim }{ $auto ->
        [yes] ，重新着色
       *[no] { "" }
    }。
settings-wallpapers-device-hint = 自动依次尝试 CUDA、WebGPU、CPU
settings-wallpapers-memory-hint = 超出上限时，自动改用 CPU 重试
