settings-system-power-desc = পিকার ও ওয়ালপেপারের ব্যাটারি সীমা আলাদা।
settings-system-power-say = ব্যাটারিতে { $cap ->
        [none] পিকারের কোনো অতিরিক্ত সীমা নেই ({ $fps })
       *[cap] পিকার { $fps }-এ সীমিত থাকে
    }{ $idle ->
        [off] , শুধু সাধারণ নিষ্ক্রিয় বিরতি প্রযোজ্য ({ $seconds })
       *[on] , কোনো ইনপুট না থাকলে { $seconds } পরে ভিডিও থামে
    }, এবং { $detail ->
        [full] ওয়ালপেপার পূর্ণ খুঁটিনাটি রাখে
       *[performance] ওয়ালপেপার পারফরম্যান্স মোডে যায়
    }।
settings-system-power-off = বন্ধ। ব্যাটারিতে পিকার ও ওয়ালপেপার চার্জারে থাকার মতোই একই সীমায় চলে।
settings-system-power-source = বর্তমান পাওয়ার উৎস: { $source }
settings-system-always = সবসময়
settings-system-on-battery = ব্যাটারিতে · সেভার চালু থাকলে
settings-system-max-fps-hint = ডিসপ্লের রিফ্রেশ রেট, বা ল্যাপটপ ঠান্ডা রাখতে 60
settings-system-idle-hint = 0 বন্ধ করে
settings-system-battery-fps-hint = এটি ও সর্বোচ্চ FPS-এর মধ্যে যেটি কম · 0 বন্ধ করে
settings-system-battery-idle-hint = ছোটটি কার্যকর হয় · 0 বন্ধ করে
settings-system-battery-wallpaper-hint = সরাসরি ভিডিও, 30 fps দৃশ্য
settings-system-gpu-desc = পরিবর্তনের পরে পিকার আবার খুলুন।
settings-system-gpu-say = { $mode ->
        [low] পিকার { $gpu } ব্যবহার করে: এটি ইন্টিগ্রেটেড GPU চায়।
        [high] পিকার { $gpu } ব্যবহার করে: এটি ডিসক্রিট GPU চায়।
        [none] পিকার { $gpu } ব্যবহার করে: ড্রাইভার সিদ্ধান্ত নেয়।
        [auto] পিকার { $gpu } ব্যবহার করে: ব্যাটারি সেভার চালু থাকলে এটি কম শক্তির GPU পছন্দ করে।
       *[custom] পিকার { $gpu } ব্যবহার করে, কিন্তু মানটি চিনতে পারে না, তাই স্বয়ংক্রিয়ের মতো আচরণ করে।
    }
settings-system-language-desc = পরিবর্তন সঙ্গে সঙ্গে প্রয়োগ হয় এবং সংরক্ষিত থাকে।
settings-system-language-say = { $follow ->
        [desktop] ইন্টারফেসের ভাষা { $language }, ডেস্কটপ অনুযায়ী। ওয়ালপেপারের নাম ও ট্যাগ বদলায় না।
       *[chosen] ইন্টারফেসের ভাষা { $language }। ওয়ালপেপারের নাম ও ট্যাগ বদলায় না।
    }
settings-system-bug-title = বাগ রিপোর্ট
settings-system-bug-desc = ইস্যুতে সংযুক্ত করার জন্য একটি ফাইল।
settings-system-bug-say = ভার্সন, পরিবেশ ও সাম্প্রতিক লগ একটি ফাইলে একত্র করুন।
