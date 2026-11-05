settings-colours-source = রং কোথা থেকে আসে
settings-colours-source-desc = প্রতিটি ওয়ালপেপার অনুসরণ করুন, একটি প্যালেট রাখুন বা রং বদলাবেন না।
settings-colours-source-follow = প্রতিটি ওয়ালপেপার অনুসরণ করে
settings-colours-source-fixed = একটি নির্দিষ্ট প্যালেট থেকে আসে
settings-colours-source-off = যেমন আছে তেমন থাকে
settings-colours-source-say = রং { $policy }।
settings-colours-source-made = রং { $policy }, তৈরি করে { $authority }।
settings-colours-end4 = end-4
settings-colours-control-menu = রং নিয়ন্ত্রণ · শনাক্ত শেল
settings-colours-generator-menu = জেনারেটর · শুধু ইনস্টল করা
settings-colours-theme-menu = থিম · প্রিসেট, আপনার, কাস্টম বীজ
settings-colours-all-presets = সব { $count } প্রিসেট…
settings-colours-generator = জেনারেটর
settings-colours-generator-desc = যে প্রোগ্রাম ওয়ালপেপারকে প্যালেটে রূপ দেয়।
settings-colours-generator-say = { $engine } প্যালেট বানায়।
settings-colours-generator-seeded = { $engine } প্যালেট বানায়, বীজ স্লট { $index }, কনট্রাস্ট { $contrast }।
settings-colours-generator-shell = { $shell } ওয়ালপেপার থেকে প্যালেট বানায়।
settings-colours-palette = প্যালেট
settings-colours-palette-desc = সব ওয়ালপেপারের জন্য একটি প্যালেট।
settings-colours-palette-say = সব ওয়ালপেপারের জন্য { $theme }।
settings-colours-idle = রং অপরিবর্তিত থাকলে ব্যবহার হয় না।
settings-colours-index-hint = কোন প্যালেট স্লট বীজ রং হবে
settings-colours-contrast-hint = -1 থেকে 1
settings-colours-wallust-hint = wallust প্যালেট বাধ্য করুন, যেমন dark16। খালি: মোড অনুযায়ী
settings-colours-seeds-hint = কাস্টম বীজ এটি ব্যবহার করে
settings-colours-config-hint = বাইরের Matugen
settings-colours-command-hint = %config% %path% %scheme% %mode% %index%
settings-colours-auto = স্বয়ংক্রিয়
settings-colours-look = চেহারা
settings-colours-look-desc = রং কতটা ছড়ায়, ফিনিশ, এবং হালকা বা গাঢ়।
settings-colours-look-say = { $scheme } ধরন, { $finish } ফিনিশ, ভ্যারিয়েন্ট { $variant }।
settings-colours-look-imported = { $shell } এর প্যালেট, { $variant } ভ্যারিয়েন্ট হিসেবে আমদানি।
settings-colours-look-shell = { $shell } নিজের চেহারা ঠিক করে।
settings-colours-look-fixed = নির্দিষ্ট প্যালেট নিজের চেহারা ঠিক করে।
settings-colours-no-designer = থিম ডিজাইনার ওয়ালপেপার অনুসরণ বা নির্দিষ্ট প্যালেটে পাওয়া যায়।
settings-colours-designer-hint = হাতে প্যালেট বদলান বা বানান
settings-colours-go = রং কোথায় যায়
settings-colours-go-desc = প্যালেট বদলালে যে অ্যাপ থিম অনুসরণ করে।
settings-colours-find-hint = অ্যাপ ইনস্টলের পর আবার দেখুন
settings-colours-matugen-hint = Matugen আউটপুটের সুইচ
settings-colours-reload-hint = ফাইল লেখার পর চলে
settings-colours-output-heading = আউটপুট { $count }
settings-colours-shells = শেল
settings-colours-shells-desc = Noctalia ও Dank Material Shell Skwd এর রং নিতে পারে।
settings-colours-dms-hint = DMS কে Auto (ওয়ালপেপার) ব্যবহার করতে হবে
settings-colours-mode-hint = প্রতিটি রং স্কিমের সঙ্গে প্রযোজ্য
settings-colours-bin-hint = PATH এ না থাকলে
settings-colours-lock = লক স্ক্রিন
settings-colours-lock-desc = যে লক স্ক্রিন আপনার ওয়ালপেপার দেখায়।
settings-colours-lock-follow = শেষ প্রয়োগ করা ওয়ালপেপার
settings-colours-lock-still = একটি স্থির ছবি
settings-colours-sddm-hint = প্রশাসকের অনুমোদন দরকার
settings-colours-go-say = { $val }{ $all } টির মধ্যে { $on } টি অ্যাপ{ $end } { $followers ->
        [one] আপনার রং
       *[some] আপনার রং
    } অনুসরণ করে{ $changed ->
        [one] ; { $val }{ $name } এর কনফিগ Skwd এর বাইরে বদলেছে{ $end }।
        [some] ; { $val }{ $count } টি অ্যাপ Skwd এর বাইরে বদলেছে{ $end }।
       *[none] ।
    }
settings-colours-outputs-say = { $state ->
        [none] এখনও কোনো ইন্টিগ্রেশন টেমপ্লেট বানায় না।
        [single] { $val }1 টি ইন্টিগ্রেশন{ $end } প্রতিবার রং বদলের পর টেমপ্লেট ফাইলে লেখে।
       *[many] { $val }{ $count } টি ইন্টিগ্রেশন{ $end } প্রতিবার রং বদলের পর টেমপ্লেট ফাইলে লেখে।
    }
settings-colours-shells-say = Noctalia { $val }{ $noctalia ->
        [follows] Skwd এর রং অনুসরণ করে
       *[keeps] নিজের রং রাখে
    }{ $end }; DMS { $val }{ $dms ->
        [follows] Skwd এর রং অনুসরণ করে
       *[keeps] নিজের রং রাখে
    }{ $end }{ $caelestia ->
        [yes] ; Caelestia ওয়ালপেপার সিঙ্ক হয়।
       *[no] ।
    }
settings-colours-lock-say = { $state ->
        [none] কোনো লক স্ক্রিন সেট নেই। নতুনগুলো { $mode } ব্যবহার করে।
        [one] { $names } { $mode } ব্যবহার করে।
       *[some] { $names } { $mode } ব্যবহার করে।
    }
settings-colours-lock-say-static = { $state ->
        [none] কোনো লক স্ক্রিন সেট নেই। নতুনগুলো { $mode } ব্যবহার করে: { $image }।
        [one] { $names } { $mode } ব্যবহার করে: { $image }।
       *[some] { $names } { $mode } ব্যবহার করে: { $image }।
    }
settings-menu-more = আরও
