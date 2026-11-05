settings-colours-source = مصدر اللون
settings-colours-source-desc = اتبع كل خلفية، أو أبقِ لوحة واحدة، أو اترك الألوان كما هي.
settings-colours-source-follow = تتبع كل خلفية
settings-colours-source-fixed = تأتي من لوحة ثابتة
settings-colours-source-off = تبقى كما هي
settings-colours-source-say = الألوان { $policy }.
settings-colours-source-made = الألوان { $policy }، ينشئها { $authority }.
settings-colours-end4 = end-4
settings-colours-control-menu = التحكم في اللون · الواجهات المكتشفة
settings-colours-generator-menu = المولّد · المثبت فقط
settings-colours-theme-menu = السمة · الجاهزة، سماتك، البذور المخصصة
settings-colours-all-presets = كل القوالب الجاهزة ({ $count })…
settings-colours-generator = المولّد
settings-colours-generator-desc = البرنامج الذي يحوّل الخلفية إلى لوحة.
settings-colours-generator-say = ينشئ { $engine } اللوحة.
settings-colours-generator-seeded = ينشئ { $engine } اللوحة، ببذرة من الخانة { $index } وتباين { $contrast }.
settings-colours-generator-shell = ينشئ { $shell } اللوحة من الخلفية.
settings-colours-palette = اللوحة
settings-colours-palette-desc = لوحة واحدة لكل الخلفيات.
settings-colours-palette-say = لكل الخلفيات { $theme }.
settings-colours-idle = غير مستخدم ما دامت الألوان كما هي.
settings-colours-index-hint = خانة اللوحة المستخدمة كلون بذرة
settings-colours-contrast-hint = من -1 إلى 1
settings-colours-wallust-hint = فرض لوحة wallust، مثل dark16. الفراغ يتبع الوضع
settings-colours-seeds-hint = تستخدمها البذور المخصصة
settings-colours-config-hint = برنامج Matugen خارجي
settings-colours-command-hint = ‎%config% %path% %scheme% %mode% %index%‎
settings-colours-auto = تلقائي
settings-colours-look = المظهر
settings-colours-look-desc = مدى انتشار اللون وتشطيبه وفاتح أم داكن.
settings-colours-look-say = طابع { $scheme } بتشطيب { $finish }، النوع { $variant }.
settings-colours-look-imported = لوحة { $shell }، مستوردة بالنوع { $variant }.
settings-colours-look-shell = يحدد { $shell } مظهره بنفسه.
settings-colours-look-fixed = تحدد اللوحة الثابتة مظهرها بنفسها.
settings-colours-no-designer = مصمم السمات متاح مع اتباع الخلفية أو اللوحة الثابتة.
settings-colours-designer-hint = اضبط لوحة أو أنشئها يدويًا
settings-colours-go = إلى أين تذهب الألوان
settings-colours-go-desc = سمات التطبيقات التي تتبع اللوحة عند تغيرها.
settings-colours-find-hint = تحقق مجددًا بعد تثبيت تطبيق
settings-colours-matugen-hint = مفتاح مخرجات Matugen
settings-colours-reload-hint = يعمل بعد كتابة الملف
settings-colours-output-heading = المخرج { $count }
settings-colours-shells = الواجهات
settings-colours-shells-desc = يمكن لبرنامجي Noctalia وDank Material Shell استخدام ألوان Skwd.
settings-colours-dms-hint = يجب أن يستخدم DMS وضع Auto (الخلفية)
settings-colours-mode-hint = يُطبَّق مع كل نظام ألوان
settings-colours-bin-hint = إن لم يكن في PATH
settings-colours-lock = شاشات القفل
settings-colours-lock-desc = شاشات القفل التي تعرض خلفيتك.
settings-colours-lock-follow = آخر خلفية مطبقة
settings-colours-lock-still = صورة ثابتة
settings-colours-sddm-hint = يتطلب موافقة المسؤول
settings-colours-go-say = { $val }{ $on } من { $all } تطبيقات{ $end } { $followers ->
        [one] يتبع
       *[some] تتبع
    } ألوانك{ $changed ->
        [one] ؛ { $val }تغيّر إعداد { $name } خارج Skwd{ $end }.
        [some] ؛ { $val }تغيّر { $count } تطبيقات خارج Skwd{ $end }.
       *[none] .
    }
settings-colours-outputs-say = { $state ->
        [none] لا يوجد تكامل يعرض قالبًا بعد.
        [single] { $val }تكامل واحد{ $end } يكتب قالبًا إلى ملف بعد كل تغيير لون.
       *[many] { $val }{ $count } تكاملات{ $end } تكتب قوالب إلى ملفات بعد كل تغيير لون.
    }
settings-colours-shells-say = برنامج Noctalia { $val }{ $noctalia ->
        [follows] يتبع ألوان Skwd
       *[keeps] يحتفظ بألوانه
    }{ $end }؛ برنامج DMS { $val }{ $dms ->
        [follows] يتبع ألوان Skwd
       *[keeps] يحتفظ بألوانه
    }{ $end }{ $caelestia ->
        [yes] ؛ تتزامن خلفيات Caelestia.
       *[no] .
    }
settings-colours-lock-say = { $state ->
        [none] لم تُضبط أي شاشة قفل. تستخدم الجديدة { $mode }.
        [one] يستخدم { $names } { $mode }.
       *[some] تستخدم { $names } { $mode }.
    }
settings-colours-lock-say-static = { $state ->
        [none] لم تُضبط أي شاشة قفل. تستخدم الجديدة { $mode }: { $image }.
        [one] يستخدم { $names } { $mode }: { $image }.
       *[some] تستخدم { $names } { $mode }: { $image }.
    }
settings-menu-more = المزيد
