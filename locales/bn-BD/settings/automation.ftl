settings-automation-rotation-desc = প্রতিবার সময়সীমা শেষ হলে পুল থেকে একটি এলোমেলো ওয়ালপেপার প্রয়োগ করে।
settings-automation-rotation-say = প্রতি { $interval } অন্তর, { $val }{ $pool }{ $end } থেকে একটি এলোমেলো বাছাই।
settings-automation-rotation-off = বন্ধ। আপনি নিজে বাছাই করলেই কেবল ওয়ালপেপার বদলায়।
settings-automation-pool = { $count ->
        [none] একটি খালি পুল
        [one] { $first }
        [two] { $first } ও { $second }
       *[three] { $first }, { $second } ও { $third }
    }
settings-automation-pool-favourites = { $favourites ->
        [only] প্রিয় { $pool }
       *[all] { $pool }
    }
settings-automation-pool-images = ছবি
settings-automation-pool-videos = ভিডিও
settings-automation-pool-engine = Wallpaper Engine আইটেম
settings-automation-pool-heading = পুলে আছে
settings-automation-timing = সময়
settings-automation-interval-hint = কমপক্ষে 10 সে
settings-automation-schedule-desc = নিয়মগুলো সময়, সপ্তাহের দিন, আবহাওয়া, পাওয়ার ও ডিসপ্লে অনুযায়ী ওয়ালপেপার ও থিম বাছাই করে। প্রথম মিলে যাওয়া নিয়মটি কার্যকর হয়।
settings-automation-schedule-say = { $start ->
        [applies] নিয়ম চালু আছে; প্রথম মিলে যাওয়া নিয়ম { $val }শুরুর সময় প্রয়োগ হয়{ $end }। আপনার নিজের বাছাই করা ওয়ালপেপার তবুও অগ্রাধিকার পায়।
       *[waits] নিয়ম চালু আছে; প্রথম মিলে যাওয়া নিয়ম { $val }পরের পরিবর্তনের অপেক্ষা করে{ $end }। আপনার নিজের বাছাই করা ওয়ালপেপার তবুও অগ্রাধিকার পায়।
    }
settings-automation-schedule-off = বন্ধ। আপনি এটি চালু না করা পর্যন্ত নিয়মগুলো রাখা থাকে।
settings-automation-start-hint = সার্ভিস শুরুর সময় প্রথম মিলে যাওয়া নিয়ম
settings-automation-editor-hint = শর্তের ব্লক দিয়ে নিয়ম তৈরি করুন
settings-automation-location = অবস্থান ও আবহাওয়া
settings-automation-location-desc = সূর্যোদয় ও সূর্যাস্তের সময় এবং আবহাওয়ার অবস্থার জন্য ব্যবহৃত হয়।
settings-automation-location-say = { $city }-এর আবহাওয়া, { $lat }, { $lon }-এ সূর্যের সময়। আবহাওয়া অনুযায়ী পরিবর্তন: { $mode }।
settings-automation-no-city = কোনো শহর নেই
settings-automation-no-latitude = কোনো অক্ষাংশ নেই
settings-automation-no-longitude = কোনো দ্রাঘিমাংশ নেই
settings-automation-location-hint = স্থানাঙ্ক ব্যবহার করতে শহর খালি রাখুন। অক্ষাংশ উত্তরে ধনাত্মক, দ্রাঘিমাংশ পূর্বে ধনাত্মক।
settings-automation-startup = শুরু ও বিজ্ঞপ্তি
settings-automation-startup-desc = সার্ভিস শুরু হলে এবং ওয়ালপেপার বদলালে কী হয়।
settings-automation-startup-say = { $restore ->
        [restores] শুরুর সময় Skwd { $val }শেষ ওয়ালপেপারটি আবার প্রয়োগ করে{ $end }।
       *[leaves] শুরুর সময় Skwd { $val }ডেস্কটপ যেমন আছে তেমন রাখে{ $end }।
    } { $notify ->
        [notifies] ওয়ালপেপার বদলালে { $val }বিজ্ঞপ্তি পাঠানো হয়{ $end }।
       *[silent] ওয়ালপেপার { $val }নীরবে বদলায়{ $end }।
    }
settings-automation-commands = ওয়ালপেপার প্রয়োগের পরে
settings-automation-commands-desc = প্রতিটি পরিবর্তনের পরে নিজের কমান্ড চালান, যেমন অন্য কোনো ওয়ালপেপার টুল চালাতে।
settings-automation-command-count = { $count ->
        [one] { $val }{ $count }টি কমান্ড{ $end } প্রতিটি পরিবর্তনের পরে চলে।
       *[other] { $val }{ $count }টি কমান্ড{ $end } প্রতিটি পরিবর্তনের পরে চলে।
    }
settings-automation-commands-say = { $listed ->
        [none] প্রতিটি পরিবর্তনের পরে { $val }কোনো কমান্ড{ $end } চলে না।
       *[some] { $commands }
    } { $apply ->
        [commands] { $rerun ->
            [also] Skwd { $val }প্রয়োগের কাজ আপনার কমান্ডের ওপর ছেড়ে দেয়{ $end }, আর কমান্ডগুলো { $val }শুরুর পুনরুদ্ধারেও চলে{ $end }।
           *[skip] Skwd { $val }প্রয়োগের কাজ আপনার কমান্ডের ওপর ছেড়ে দেয়{ $end }, আর কমান্ডগুলো { $val }শুরুর পুনরুদ্ধার এড়িয়ে যায়{ $end }।
        }
       *[itself] { $rerun ->
            [also] Skwd { $val }নিজেই ওয়ালপেপার প্রয়োগ করে{ $end }, আর কমান্ডগুলো { $val }শুরুর পুনরুদ্ধারেও চলে{ $end }।
           *[skip] Skwd { $val }নিজেই ওয়ালপেপার প্রয়োগ করে{ $end }, আর কমান্ডগুলো { $val }শুরুর পুনরুদ্ধার এড়িয়ে যায়{ $end }।
        }
    }
settings-automation-pick-only-hint = আপনার কমান্ড এটি প্রয়োগ করে
settings-automation-commands-heading = কমান্ড
settings-automation-placeholders = কমান্ডে %path%, %thumb%, %type% ও %name% ব্যবহার করুন।
settings-automation-minutes = { $count } মিনিট
settings-automation-mode-off = বন্ধ
settings-automation-mode-outdoors = বাইরের আবহাওয়া অনুসরণ
settings-automation-mode-playlists = আপনার প্লেলিস্ট থেকে বাছাই
settings-automation-mode-semantic = আপনার লাইব্রেরিতে খোঁজা
settings-automation-condition-clear = পরিষ্কার
settings-automation-condition-cloudy = মেঘলা
settings-automation-condition-rainy = বৃষ্টি
settings-automation-condition-snowy = তুষার
settings-automation-condition-stormy = ঝড়
settings-automation-condition-foggy = কুয়াশা
settings-automation-condition-windy = বাতাস
settings-automation-display-another = আরেকটি
settings-automation-display-exclude = বাদ দিন
settings-automation-display-keep = রাখুন
settings-automation-playlists-heading = আবহাওয়া অনুযায়ী প্লেলিস্ট
settings-automation-hours = { $count } ঘণ্টা
