settings-system-power-desc = انتخابگر و تصاویر زمینه محدودیت‌های باتری جداگانه دارند.
settings-system-power-say = روی باتری { $cap ->
        [none] انتخابگر محدودیت اضافه‌ای ندارد ({ $fps })
       *[cap] انتخابگر به { $fps } محدود می‌شود
    }{ $idle ->
        [off] ، فقط مکث معمولی هنگام بیکاری اعمال می‌شود ({ $seconds })
       *[on] ، ویدیو پس از { $seconds } بدون ورودی متوقف می‌شود
    } و { $detail ->
        [full] تصاویر زمینه همهٔ جزئیات را نگه می‌دارند
       *[performance] تصاویر زمینه به حالت عملکرد می‌روند
    }.
settings-system-power-off = خاموش. روی باتری، انتخابگر و تصاویر زمینه با همان محدودیت‌های حالت وصل به برق کار می‌کنند.
settings-system-power-source = منبع برق فعلی: { $source }
settings-system-always = همیشه
settings-system-on-battery = روی باتری · وقتی ذخیره‌ساز روشن است
settings-system-max-fps-hint = نرخ تازه‌سازی نمایشگر، یا 60 برای لپ‌تاپی خنک‌تر
settings-system-idle-hint = 0 خاموش می‌کند
settings-system-battery-fps-hint = کمترینِ این و بیشینهٔ FPS · 0 خاموش می‌کند
settings-system-battery-idle-hint = کوتاه‌تر اعمال می‌شود · 0 خاموش می‌کند
settings-system-battery-wallpaper-hint = ویدیوی مستقیم، صحنه‌های 30 fps
settings-system-gpu-desc = پس از تغییر، انتخابگر را دوباره باز کنید.
settings-system-gpu-say = { $mode ->
        [low] انتخابگر از { $gpu } استفاده می‌کند: GPU مجتمع درخواست می‌کند.
        [high] انتخابگر از { $gpu } استفاده می‌کند: GPU مجزا درخواست می‌کند.
        [none] انتخابگر از { $gpu } استفاده می‌کند: درایور تصمیم می‌گیرد.
        [auto] انتخابگر از { $gpu } استفاده می‌کند: وقتی ذخیرهٔ باتری فعال است، GPU کم‌مصرف را ترجیح می‌دهد.
       *[custom] انتخابگر از { $gpu } استفاده می‌کند، اما این مقدار را نمی‌شناسد، پس مثل خودکار رفتار می‌کند.
    }
settings-system-language-desc = تغییرها فوراً اعمال و ذخیره می‌شوند.
settings-system-language-say = { $follow ->
        [desktop] زبان رابط { $language } است، مطابق میزکار. نام و برچسب تصاویر زمینه تغییر نمی‌کند.
       *[chosen] زبان رابط { $language } است. نام و برچسب تصاویر زمینه تغییر نمی‌کند.
    }
settings-system-bug-title = گزارش خطا
settings-system-bug-desc = یک فایل برای پیوست به گزارش مشکل.
settings-system-bug-say = نسخه، محیط و گزارش‌های اخیر را در یک فایل جمع کنید.
