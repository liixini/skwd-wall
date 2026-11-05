settings-library-folders-desc = افحص هذه المجلدات ومجلداتها الفرعية بحثًا عن الصور والفيديو.
settings-library-folders-say = جارٍ فحص { $val }{ $folders }{ $end }.{ $state ->
        [polling] { " " }الاستطلاع يغطي ما تفوته المراقبة الأصلية، كل { $every }.{ $unread }
        [native] { " " }مراقبة الملفات الأصلية نشطة؛ الاستطلاع متوقف.
        [down] { " " }فشلت المراقبة الأصلية والاستطلاع معطل.
       *[unknown] { "" }
    }
settings-library-unread = { $count ->
        [one] { " " }تعذرت قراءة مجلد واحد.
       *[other] { " " }تعذرت قراءة { $count } مجلدات.
    }
settings-library-watching = المراقبة
settings-library-poll-hint = لمجلدات الشبكة أو FUSE
settings-library-interval-hint = أعد تشغيل skwd-walld بعد التغيير
settings-library-excluded-desc = تخطَّ هذه المجلدات ومجلداتها الفرعية. تبقى البيانات الوصفية محفوظة.
settings-library-excluded-say = لا يُفحص أبدًا: { $val }{ $folders }{ $end }.{ $outside }
settings-library-outside = { $count ->
        [one] { " " }واحد خارج كل مجلدات الخلفيات ولا يتخطى شيئًا.
       *[other] { " " }{ $count } خارج كل مجلدات الخلفيات ولا تتخطى شيئًا.
    }
settings-library-excluded-hint = المجلدات المستبعدة لا تُفحص أبدًا، فلا يمكن لأي ملف شخصي إظهارها. للاحتفاظ بمجلد وإخفائه أمام الآخرين استخدم { $link }.
settings-library-profiles-desc = حدد المجلدات التي يستخدمها المنتقي ومرشح المجلدات والخلفيات العشوائية والجداول.
settings-library-profiles-say = الملف الشخصي النشط { $state ->
        [none] { $menu }، ويعرض كل المجلدات.
       *[some] { $menu }:‏ { $summary }.
    } Ctrl+H أو زر المخفي في الشريط يعرض الخلفيات المخفية، ومنها المخفية من بطاقتها، حتى التشغيل التالي.
settings-library-active = الملف الشخصي النشط
settings-library-terminal = التبديل من الطرفية
settings-library-tree-note = الإخفاء يبقي خارج العشوائي حتى Ctrl+H. الاستبعاد يبقي خارجًا حتى مع Ctrl+H. المجلد يضبط مجلداته الفرعية.
settings-library-show-tip = في المنتقي والعشوائي
settings-library-hide-tip = حتى Ctrl+H، لا عشوائي
settings-library-exclude-tip = خارج هذا الملف، يتخطاه Ctrl+H
settings-library-from = من { $link }
settings-library-excluded-inside = { $count } مستبعدة بالداخل
settings-library-both-inside = { $hidden } مخفية، { $excluded } مستبعدة بالداخل
settings-library-storage = التخزين والمهملات
settings-library-storage-desc = نسخ WebP أصغر وتخزين للاستعادة ونسخ معاد تلوينها.
settings-library-storage-say = { $optimise ->
        [on] تصبح الصور الجديدة WebP بجودة { $quality } وبحد { $size }.
       *[off] تبقى الصور الجديدة كما هي. تحسين الكل يستخدم جودة { $quality } وبحد { $size }.
    } { $trash ->
        [on] تبقى الأصول في المهملات { $days } ثم تُحذف.
       *[off] تبقى الأصول في المهملات حتى تحذفها.
    } { $recolour ->
        [on] تحصل الخلفيات الجديدة أيضًا على نسخة معاد تلوينها.
       *[off] لا يعاد تلوين الخلفيات الجديدة.
    }
settings-library-retention-hint = أيام الاحتفاظ بالصور في المهملات
settings-library-optimise-hint = حسّن كل صور المكتبة
settings-library-recolour = إعادة التلوين
settings-library-thumbnails = الصور المصغرة والإصلاح
settings-library-thumbnails-desc = أعد بناء المعاينات وبيانات الألوان عندما يبدو شيء خاطئًا.
settings-library-thumbnails-say = تُنشأ حتى { $jobs } صور مصغرة معًا أثناء إعادة البناء.
settings-library-rebuild = إعادة البناء
settings-library-start-over = البدء من جديد
settings-library-recompute-hint = من الصور المصغرة الحالية
settings-library-clear-hint = تُعاد الصور المصغرة في الفحص التالي
