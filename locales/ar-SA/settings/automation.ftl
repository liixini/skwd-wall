settings-automation-rotation-desc = يطبّق خلفية عشوائية من المجموعة كلما انتهت الفترة.
settings-automation-rotation-say = كل { $interval }، اختيار عشوائي من { $val }{ $pool }{ $end }.
settings-automation-rotation-off = متوقف. لا تتغير الخلفية إلا عندما تختار واحدة بنفسك.
settings-automation-pool = { $count ->
        [none] مجموعة فارغة
        [one] { $first }
        [two] { $first } و{ $second }
       *[three] { $first } و{ $second } و{ $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] { $pool } المفضلة
       *[all] { $pool }
    }
settings-automation-pool-images = الصور
settings-automation-pool-videos = الفيديوهات
settings-automation-pool-engine = عناصر Wallpaper Engine
settings-automation-pool-heading = تضم المجموعة
settings-automation-timing = التوقيت
settings-automation-interval-hint = 10 ث على الأقل
settings-automation-schedule-desc = تختار القواعد الخلفية والسمة حسب الوقت ويوم الأسبوع والطقس والطاقة والشاشات. تفوز أول قاعدة مطابقة.
settings-automation-schedule-say = { $start ->
        [applies] القواعد مفعّلة؛ أول قاعدة مطابقة { $val }تُطبَّق عند البدء{ $end }. الخلفية التي تختارها بنفسك تبقى لها الأولوية.
       *[waits] القواعد مفعّلة؛ أول قاعدة مطابقة { $val }تنتظر التغيير التالي{ $end }. الخلفية التي تختارها بنفسك تبقى لها الأولوية.
    }
settings-automation-schedule-off = متوقف. تبقى قواعدك محفوظة حتى تفعّل هذا.
settings-automation-start-hint = أول قاعدة مطابقة عند بدء الخدمة
settings-automation-editor-hint = ابنِ القواعد من كتل الشروط
settings-automation-location = الموقع والطقس
settings-automation-location-desc = يُستخدم لأوقات الشروق والغروب ولأحوال الطقس.
settings-automation-location-say = الطقس في { $city }، وأوقات الشمس عند { $lat }، { $lon }. التغييرات حسب الطقس: { $mode }.
settings-automation-no-city = بلا مدينة
settings-automation-no-latitude = بلا خط عرض
settings-automation-no-longitude = بلا خط طول
settings-automation-location-hint = اترك المدينة فارغة لاستخدام الإحداثيات بدلًا منها. خط العرض موجب شمالًا وخط الطول موجب شرقًا.
settings-automation-startup = البدء والإشعارات
settings-automation-startup-desc = ما يحدث عند بدء الخدمة وعند تغيّر الخلفية.
settings-automation-startup-say = { $restore ->
        [restores] عند البدء { $val }يعيد Skwd تطبيق آخر خلفية{ $end }.
       *[leaves] عند البدء { $val }يترك Skwd سطح المكتب كما هو{ $end }.
    } { $notify ->
        [notifies] تغييرات الخلفية { $val }ترسل إشعارًا{ $end }.
       *[silent] تغييرات الخلفية { $val }تبقى صامتة{ $end }.
    }
settings-automation-commands = بعد تطبيق خلفية
settings-automation-commands-desc = شغّل أوامرك الخاصة بعد كل تغيير، مثلًا لتشغيل أداة خلفيات أخرى.
settings-automation-command-count = { $count ->
        [zero] { $val }لا أوامر{ $end } تعمل بعد كل تغيير.
        [one] { $val }أمر واحد{ $end } يعمل بعد كل تغيير.
        [two] { $val }أمران{ $end } يعملان بعد كل تغيير.
        [few] { $val }{ $count } أوامر{ $end } تعمل بعد كل تغيير.
        [many] { $val }{ $count } أمرًا{ $end } يعمل بعد كل تغيير.
       *[other] { $val }{ $count } أمر{ $end } يعمل بعد كل تغيير.
    }
settings-automation-commands-say = { $listed ->
        [none] { $val }لا تعمل أي أوامر{ $end } بعد كل تغيير.
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] { $val }يترك Skwd التطبيق لأوامرك{ $end }، و{ $val }تعمل الأوامر أيضًا عند الاستعادة في البدء{ $end }.
           *[skip] { $val }يترك Skwd التطبيق لأوامرك{ $end }، و{ $val }تتخطى الأوامر الاستعادة عند البدء{ $end }.
        }
       *[itself] { $rerun ->
            [also] { $val }يطبّق Skwd الخلفية بنفسه{ $end }، و{ $val }تعمل الأوامر أيضًا عند الاستعادة في البدء{ $end }.
           *[skip] { $val }يطبّق Skwd الخلفية بنفسه{ $end }، و{ $val }تتخطى الأوامر الاستعادة عند البدء{ $end }.
        }
    }
settings-automation-pick-only-hint = أوامرك تطبّقها
settings-automation-commands-heading = الأوامر
settings-automation-placeholders = استخدم %path% و%thumb% و%type% و%name% في الأمر.
settings-automation-minutes = { $count } د
settings-automation-mode-off = متوقف
settings-automation-mode-outdoors = اتبع الطقس في الخارج
settings-automation-mode-playlists = اختر من قوائم التشغيل
settings-automation-mode-semantic = ابحث في مكتبتك
settings-automation-condition-clear = صافٍ
settings-automation-condition-cloudy = غائم
settings-automation-condition-rainy = ممطر
settings-automation-condition-snowy = مثلج
settings-automation-condition-stormy = عاصف
settings-automation-condition-foggy = ضبابي
settings-automation-condition-windy = رياح
settings-automation-display-another = أخرى
settings-automation-display-exclude = استبعاد
settings-automation-display-keep = إبقاء
settings-automation-playlists-heading = قائمة لكل طقس
settings-automation-hours = { $count } س
