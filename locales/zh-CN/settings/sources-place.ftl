settings-sources-other = 其他来源
settings-sources-other-desc = Unsplash、Pexels、必应每日图片和 YouTube。
settings-sources-wallhaven-say = { $top ->
        [yes] 新会话按 { $sort } 排序，时间 { $range }，比例 { $ratio }，显示 { $categories }，内容 { $purity }。
       *[no] 新会话按 { $sort } 排序，比例 { $ratio }，显示 { $categories }，内容 { $purity }。
    }
settings-sources-wallhaven-off = 已关闭。Wallhaven 浏览器已隐藏。
settings-sources-workshop-say = { $trend ->
        [yes] 新会话按 { $sort } 排序，时间 { $days }，类型 { $type }，分辨率 { $resolution }，分类 { $category }。
       *[no] 新会话按 { $sort } 排序，类型 { $type }，分辨率 { $resolution }，分类 { $category }。
    }
settings-sources-workshop-off = 已关闭。Steam 创意工坊浏览器和 Wallpaper Engine 场景设置已隐藏。
settings-sources-no-category = 无分类
settings-sources-no-content = 无内容级别
settings-sources-content = 内容
settings-sources-account = 账户
settings-sources-browser = 浏览器
settings-sources-browser-folders = 浏览器和文件夹
settings-sources-photos = 照片
settings-sources-daily = 每日图片和视频
settings-sources-not-set = 未设置
settings-sources-detected = 从 Steam 检测
settings-sources-nsfw-hint = NSFW 结果需要
settings-sources-grid-hint = 列 × 行
settings-sources-thumb-hint = 宽 × 高
settings-sources-steamcmd-hint = 用于 steamcmd
settings-sources-workshop-only = 仅创意工坊
settings-sources-method-steam = Steam 客户端
settings-sources-method-steamcmd = steamcmd
settings-sources-method-steam-hint = 需要 skwd-deck-steamworks 并运行 Steam
settings-sources-method-steamcmd-hint = 需要 steamcmd 和一次性登录
settings-sources-unsplash-hint = 在 unsplash.com/developers 免费获取
settings-sources-pexels-hint = 在 pexels.com/api 免费获取
settings-sources-bing-hint = 最近 8 张首页图片
settings-sources-youtube-hint = 需要 PATH 中有 yt-dlp
settings-sources-clip-hint = 0 表示下载全部
settings-sources-searching = 正在搜索 { $sources }。
settings-sources-searching-none = 没有来源在搜索。
settings-sources-needs-key = { $key ->
        [access] { $source } 需要访问密钥。
       *[api] { $source } 需要 API 密钥。
    }
settings-sources-off = { $count ->
        [one] { $sources } 已关闭。
       *[some] { $sources } 已关闭。
    }
