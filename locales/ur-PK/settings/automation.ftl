settings-automation-rotation-desc = جب بھی وقفہ ختم ہو، پول سے ایک بے ترتیب وال پیپر لگاتا ہے۔
settings-automation-rotation-say = ہر { $interval } بعد، { $val }{ $pool }{ $end } میں سے ایک بے ترتیب انتخاب۔
settings-automation-rotation-off = بند۔ وال پیپر صرف تب بدلتا ہے جب آپ خود چنیں۔
settings-automation-pool = { $count ->
        [none] خالی پول
        [one] { $first }
        [two] { $first } اور { $second }
       *[three] { $first }، { $second } اور { $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] پسندیدہ { $pool }
       *[all] { $pool }
    }
settings-automation-pool-images = تصاویر
settings-automation-pool-videos = ویڈیوز
settings-automation-pool-engine = Wallpaper Engine آئٹمز
settings-automation-pool-heading = پول میں شامل
settings-automation-timing = وقت
settings-automation-interval-hint = کم از کم 10 سیکنڈ
settings-automation-schedule-desc = اصول وقت، ہفتے کے دن، موسم، پاور اور ڈسپلے کے مطابق وال پیپر اور تھیم چنتے ہیں۔ پہلا مماثل اصول لاگو ہوتا ہے۔
settings-automation-schedule-say = { $start ->
        [applies] اصول آن ہیں؛ پہلا مماثل اصول { $val }آغاز پر لاگو ہوتا ہے{ $end }۔ آپ کا خود چنا ہوا وال پیپر پھر بھی ترجیح پاتا ہے۔
       *[waits] اصول آن ہیں؛ پہلا مماثل اصول { $val }اگلی تبدیلی کا انتظار کرتا ہے{ $end }۔ آپ کا خود چنا ہوا وال پیپر پھر بھی ترجیح پاتا ہے۔
    }
settings-automation-schedule-off = بند۔ جب تک آپ اسے آن نہ کریں آپ کے اصول محفوظ رہتے ہیں۔
settings-automation-start-hint = سروس شروع ہونے پر پہلا مماثل اصول
settings-automation-editor-hint = شرط کے بلاکس سے اصول بنائیں
settings-automation-location = مقام اور موسم
settings-automation-location-desc = طلوع و غروب آفتاب کے اوقات اور موسمی حالات کے لیے استعمال ہوتا ہے۔
settings-automation-location-say = { $city } کا موسم، { $lat }، { $lon } پر سورج کے اوقات۔ موسم کے مطابق تبدیلیاں: { $mode }۔
settings-automation-no-city = کوئی شہر نہیں
settings-automation-no-latitude = کوئی عرض بلد نہیں
settings-automation-no-longitude = کوئی طول بلد نہیں
settings-automation-location-hint = کوآرڈینیٹس استعمال کرنے کے لیے شہر خالی چھوڑیں۔ عرض بلد شمال کی طرف اور طول بلد مشرق کی طرف مثبت ہے۔
settings-automation-startup = آغاز اور اطلاعات
settings-automation-startup-desc = سروس شروع ہونے اور وال پیپر بدلنے پر کیا ہوتا ہے۔
settings-automation-startup-say = { $restore ->
        [restores] آغاز پر Skwd { $val }آخری وال پیپر دوبارہ لگاتا ہے{ $end }۔
       *[leaves] آغاز پر Skwd { $val }ڈیسک ٹاپ کو جیسا ہے ویسا چھوڑ دیتا ہے{ $end }۔
    } { $notify ->
        [notifies] وال پیپر بدلنے پر { $val }اطلاع بھیجی جاتی ہے{ $end }۔
       *[silent] وال پیپر { $val }خاموشی سے بدلتے ہیں{ $end }۔
    }
settings-automation-commands = وال پیپر لگنے کے بعد
settings-automation-commands-desc = ہر تبدیلی کے بعد اپنی کمانڈز چلائیں، مثلاً کسی دوسرے وال پیپر ٹول کو چلانے کے لیے۔
settings-automation-command-count = { $count ->
        [one] { $val }{ $count } کمانڈ{ $end } ہر تبدیلی کے بعد چلتی ہے۔
       *[other] { $val }{ $count } کمانڈز{ $end } ہر تبدیلی کے بعد چلتی ہیں۔
    }
settings-automation-commands-say = { $listed ->
        [none] ہر تبدیلی کے بعد { $val }کوئی کمانڈ{ $end } نہیں چلتی۔
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] Skwd { $val }لگانے کا کام آپ کی کمانڈز پر چھوڑتا ہے{ $end }، اور کمانڈز { $val }آغاز کی بحالی پر بھی چلتی ہیں{ $end }۔
           *[skip] Skwd { $val }لگانے کا کام آپ کی کمانڈز پر چھوڑتا ہے{ $end }، اور کمانڈز { $val }آغاز کی بحالی کو چھوڑ دیتی ہیں{ $end }۔
        }
       *[itself] { $rerun ->
            [also] Skwd { $val }وال پیپر خود لگاتا ہے{ $end }، اور کمانڈز { $val }آغاز کی بحالی پر بھی چلتی ہیں{ $end }۔
           *[skip] Skwd { $val }وال پیپر خود لگاتا ہے{ $end }، اور کمانڈز { $val }آغاز کی بحالی کو چھوڑ دیتی ہیں{ $end }۔
        }
    }
settings-automation-pick-only-hint = آپ کی کمانڈز اسے لگاتی ہیں
settings-automation-commands-heading = کمانڈز
settings-automation-placeholders = کمانڈ میں %path%، %thumb%، %type% اور %name% استعمال کریں۔
settings-automation-minutes = { $count } منٹ
settings-automation-mode-off = بند
settings-automation-mode-outdoors = باہر کے موسم کے مطابق
settings-automation-mode-playlists = اپنی پلے لسٹس سے چنیں
settings-automation-mode-semantic = اپنی لائبریری میں تلاش کریں
settings-automation-condition-clear = صاف
settings-automation-condition-cloudy = ابر آلود
settings-automation-condition-rainy = بارش
settings-automation-condition-snowy = برف باری
settings-automation-condition-stormy = طوفان
settings-automation-condition-foggy = دھند
settings-automation-condition-windy = تیز ہوا
settings-automation-display-another = دوسرا
settings-automation-display-exclude = خارج کریں
settings-automation-display-keep = رکھیں
settings-automation-playlists-heading = ہر موسم کی پلے لسٹ
settings-automation-hours = { $count } گھنٹے
