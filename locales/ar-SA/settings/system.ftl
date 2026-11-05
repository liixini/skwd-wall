settings-system-power-desc = للمنتقي والخلفيات حدود بطارية منفصلة.
settings-system-power-say = على البطارية { $cap ->
        [none] لا يوجد حد إضافي للمنتقي ({ $fps })
       *[cap] يُحدّ المنتقي عند { $fps }
    }{ $idle ->
        [off] ، ولا ينطبق إلا الإيقاف المؤقت العادي عند الخمول ({ $seconds })
       *[on] ، ويتوقف الفيديو بعد { $seconds } دون إدخال
    }، و{ $detail ->
        [full] تحتفظ الخلفيات بكامل التفاصيل
       *[performance] تنتقل الخلفيات إلى وضع الأداء
    }.
settings-system-power-off = متوقف. على البطارية، يعمل المنتقي والخلفيات بالحدود نفسها كما عند التوصيل بالكهرباء.
settings-system-power-source = مصدر الطاقة الحالي: { $source }
settings-system-always = دائمًا
settings-system-on-battery = على البطارية · أثناء تشغيل الموفّر
settings-system-max-fps-hint = معدل تحديث الشاشة، أو 60 لحاسوب محمول أبرد
settings-system-idle-hint = 0 يعطّله
settings-system-battery-fps-hint = الأقل بين هذا والحد الأقصى لـ FPS · 0 يعطّله
settings-system-battery-idle-hint = الأقصر هو المعتمد · 0 يعطّله
settings-system-battery-wallpaper-hint = فيديو مباشر، مشاهد بمعدل 30 fps
settings-system-gpu-desc = أعد فتح المنتقي بعد تغييره.
settings-system-gpu-say = { $mode ->
        [low] يستخدم المنتقي { $gpu }: يطلب GPU مدمجًا.
        [high] يستخدم المنتقي { $gpu }: يطلب GPU منفصلًا.
        [none] يستخدم المنتقي { $gpu }: يقرر برنامج التشغيل.
        [auto] يستخدم المنتقي { $gpu }: يفضّل GPU منخفض الطاقة أثناء تفعيل موفّر البطارية.
       *[custom] يستخدم المنتقي { $gpu }، ولا يتعرّف على هذه القيمة، لذا يعمل كأنه تلقائي.
    }
settings-system-language-desc = تُطبَّق التغييرات فورًا وتُحفظ.
settings-system-language-say = { $follow ->
        [desktop] لغة الواجهة { $language }، وهي تتبع لغة سطح المكتب. تبقى أسماء الخلفيات ووسومها كما هي.
       *[chosen] لغة الواجهة { $language }. تبقى أسماء الخلفيات ووسومها كما هي.
    }
settings-system-bug-title = تقرير خلل
settings-system-bug-desc = ملف واحد لإرفاقه ببلاغ.
settings-system-bug-say = اجمع الإصدار والبيئة والسجلات الأخيرة في ملف واحد.
