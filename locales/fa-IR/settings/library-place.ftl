settings-library-folders-desc = این پوشه‌ها و زیرپوشه‌هایشان را برای تصویر و ویدیو بررسی کن.
settings-library-folders-say = در حال بررسی { $val }{ $folders }{ $end }.{ $state ->
        [polling] { " " }نظرسنجی هر { $every } آنچه پایش بومی نمی‌بیند را پوشش می‌دهد.{ $unread }
        [native] { " " }پایش بومی فایل فعال است؛ نظرسنجی بیکار است.
        [down] { " " }پایش بومی شکست خورد و نظرسنجی خاموش است.
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }{ $count } پوشه خوانده نمی‌شود.
       *[other] { " " }{ $count } پوشه خوانده نمی‌شوند.
    }
settings-library-watching = پایش
settings-library-poll-hint = برای پوشه‌های شبکه یا FUSE
settings-library-interval-hint = پس از تغییر skwd-walld را دوباره راه بیندازید
settings-library-excluded-desc = از این پوشه‌ها و زیرپوشه‌هایشان بگذر. فراداده‌های موجود می‌مانند.
settings-library-excluded-say = هرگز بررسی نمی‌شود: { $val }{ $folders }{ $end }.{ $outside }
settings-library-outside = { $count ->
        [one] { " " }{ $count } تا بیرون از همهٔ پوشه‌های والپیپر است و چیزی را رد نمی‌کند.
       *[other] { " " }{ $count } تا بیرون از همهٔ پوشه‌های والپیپر هستند و چیزی را رد نمی‌کنند.
    }
settings-library-excluded-hint = پوشه‌های مستثنا هرگز بررسی نمی‌شوند، پس هیچ نمایه‌ای نمی‌تواند نشانشان دهد. برای نگه داشتن پوشه و پنهان کردن آن در جمع از { $link } استفاده کنید.
settings-library-profiles-desc = تعیین کنید انتخابگر، فیلتر پوشه، والپیپرهای تصادفی و زمان‌بندی‌ها از کدام پوشه‌ها استفاده کنند.
settings-library-profiles-say = نمایهٔ فعال { $state ->
        [none] { $menu }، که همهٔ پوشه‌ها را نشان می‌دهد.
       *[some] { $menu }:‏ { $summary }.
    } Ctrl+H یا دکمهٔ پنهان‌ها در نوار، والپیپرهای پنهان، حتی پنهان‌شده از کارت، را تا اجرای بعدی نشان می‌دهد.
settings-library-active = نمایهٔ فعال
settings-library-terminal = تغییر از ترمینال
settings-library-tree-note = پنهان تا Ctrl+H از تصادفی بیرون می‌ماند. مستثنا حتی با Ctrl+H بیرون می‌ماند. پوشه زیرپوشه‌هایش را هم تنظیم می‌کند.
settings-library-show-tip = در انتخابگر و تصادفی
settings-library-hide-tip = تا Ctrl+H، هرگز تصادفی نه
settings-library-exclude-tip = بیرون از این نمایه، Ctrl+H رد می‌کند
settings-library-from = از { $link }
settings-library-excluded-inside = { $count } مستثنا در داخل
settings-library-both-inside = { $hidden } پنهان، { $excluded } مستثنا در داخل
settings-library-storage = فضای ذخیره و زباله‌دان
settings-library-storage-desc = نسخه‌های کوچک‌تر WebP، فضای بازیابی و نسخه‌های رنگ‌آمیزی‌شده.
settings-library-storage-say = { $optimise ->
        [on] تصویرهای جدید با کیفیت { $quality } و سقف { $size } به WebP تبدیل می‌شوند.
       *[off] تصویرهای جدید همان‌طور می‌مانند. بهینه‌سازی همه کیفیت { $quality } و سقف { $size } را به کار می‌برد.
    } { $trash ->
        [on] اصل‌های زباله‌دان { $days } می‌مانند و سپس حذف می‌شوند.
       *[off] اصل‌های زباله‌دان تا حذف شما می‌مانند.
    } { $recolour ->
        [on] والپیپرهای جدید نسخهٔ رنگ‌آمیزی‌شده هم می‌گیرند.
       *[off] والپیپرهای جدید رنگ‌آمیزی نمی‌شوند.
    }
settings-library-retention-hint = روزهای نگه‌داری تصویرهای زباله‌دان
settings-library-optimise-hint = بهینه‌سازی همهٔ تصویرهای کتابخانه
settings-library-recolour = رنگ‌آمیزی
settings-library-thumbnails = بندانگشتی‌ها و تعمیر
settings-library-thumbnails-desc = وقتی چیزی اشتباه به نظر می‌رسد پیش‌نمایش و داده‌های رنگ را بازسازی کنید.
settings-library-thumbnails-say = در بازسازی تا { $jobs } بندانگشتی همزمان ساخته می‌شود.
settings-library-rebuild = بازسازی
settings-library-start-over = از نو
settings-library-recompute-hint = از بندانگشتی‌های موجود
settings-library-clear-hint = بندانگشتی‌ها در بررسی بعدی بازسازی می‌شوند
