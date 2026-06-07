import QtQuick
import Quickshell

ShellRoot {
  id: root
  property int resultCode: -1
  property var fails: []
  property int checks: 0

  Timer {
    interval: 1
    repeat: false
    running: root.resultCode >= 0
    onTriggered: Qt.exit(root.resultCode)
  }

  function check(area, cond, msg) {
    root.checks++
    if (!cond)
      root.fails.push(area + ": " + msg)
  }

  function eq(a, b) {
    return JSON.stringify(a) === JSON.stringify(b)
  }

  function testColor(c) {
    if (!c) return
    check("color", c.colorToHue("red") === 0, "red->0 got " + c.colorToHue("red"))
    check("color", c.colorToHue("blue") === 8, "blue->8 got " + c.colorToHue("blue"))
    check("color", c.colorToHue("navy") === 9, "navy->9 got " + c.colorToHue("navy"))
    check("color", c.colorToHue("pink") === 11, "pink->11")
    check("color", c.colorToHue("gray") === 99, "gray->99")
    check("color", c.colorToHue("RED") === 0, "case-insensitive RED->0")
    check("color", c.colorToHue("reddish") === 0, "fuzzy reddish->0 got " + c.colorToHue("reddish"))
    check("color", c.colorToHue("xyzzy") === 99, "unknown xyzzy->99 got " + c.colorToHue("xyzzy"))
    check("color", c.colorToHue("") === 0, "empty-string quirk ''->0 got " + c.colorToHue(""))
    check("color", eq(c.mergeSynonyms(["mountains", "hills"]), ["mountain"]), "mergeSynonyms mountains+hills->[mountain] got " + JSON.stringify(c.mergeSynonyms(["mountains", "hills"])))
    check("color", eq(c.mergeSynonyms(["serene", "calm"]), ["peaceful"]), "serene+calm->[peaceful]")
    check("color", eq(c.mergeSynonyms(["foo", "bar"]), ["foo", "bar"]), "no-synonym passthrough")
    check("color", eq(c.mergeSynonyms([]), []), "empty array")
    check("color", eq(c.mergeSynonyms(["city", "urban"]), ["city"]), "canonical+source dedup")
  }

  function testImage(im) {
    if (!im) return
    check("image", im.hueBucket(200, 5) === 99, "low-sat->99")
    check("image", im.hueBucket(0, 50) === 0, "hue0->0")
    check("image", im.hueBucket(350, 50) === 0, "hue350->0")
    check("image", im.hueBucket(24, 50) === 0, "hue24->0")
    check("image", im.hueBucket(25, 50) === 1, "hue25->1 got " + im.hueBucket(25, 50))
    check("image", im.hueBucket(55, 50) === 2, "hue55->2 got " + im.hueBucket(55, 50))
    check("image", im.hueBucket(339, 50) === 11, "hue339->11 got " + im.hueBucket(339, 50))
    check("image", im.hueBucket(340, 50) === 0, "hue340->0 wraps")
    check("image", im.hueBucket(100, 10) === 3, "sat exactly 10 not mono got " + im.hueBucket(100, 10))
    check("image", im.smallThumbPath("/c/thumbs/x.webp") === "/c/thumbs-sm/x.webp", "thumbs-sm")
    check("image", im.smallThumbPath("/c/we-thumbs/y.webp") === "/c/thumbs-sm/we-y.webp", "we-thumbs prefix")
    check("image", im.smallThumbPath("/c/video-thumbs/z.webp") === "/c/thumbs-sm/vid-z.webp", "video-thumbs prefix")
    check("image", im.fileUrl("") === "", "empty fileUrl")
    check("image", im.fileUrl("/a b/c.png") === "file:///a%20b/c.png", "fileUrl escapes space got " + im.fileUrl("/a b/c.png"))
    check("image", im.thumbKey("x.jpg") === "x", "thumbKey jpg")
    check("image", im.thumbKey("x.webp") === "x", "thumbKey webp")
    check("image", im.thumbKey("/p/y.webp") === "y", "thumbKey path")
    check("image", im.thumbKey("noext") === "noext", "thumbKey no-ext")
    check("image", im.thumbKey("", "fallback") === "fallback", "thumbKey fallback")
    check("image", im.thumbKey("", "") === "", "thumbKey empty")
    check("image", im.findExtPattern(["jpg", "png"]) === '\\( -iname "*.jpg" -o -iname "*.png" \\)', "findExtPattern got " + im.findExtPattern(["jpg", "png"]))
  }

  function testMeta(m) {
    if (!m) return
    check("meta", m.formatSize(0) === "0 B", "0 B")
    check("meta", m.formatSize(512) === "512 B", "512 B")
    check("meta", m.formatSize(1023) === "1023 B", "1023 B")
    check("meta", m.formatSize(1024) === "1 KB", "1 KB got " + m.formatSize(1024))
    check("meta", m.formatSize(2048) === "2 KB", "2 KB")
    check("meta", m.formatSize(1048576) === "1.0 MB", "1.0 MB got " + m.formatSize(1048576))
    check("meta", m.formatSize(1073741824) === "1.0 GB", "1.0 GB got " + m.formatSize(1073741824))
    check("meta", m.formatExt("photo.JPG") === "JPG", "ext uppercase")
    check("meta", m.formatExt("a.tar.gz") === "GZ", "ext last segment")
    check("meta", m.formatExt("noext") === "", "ext none")
    check("meta", m.formatExt(".bashrc") === "", "ext dotfile->empty got " + m.formatExt(".bashrc"))
    check("meta", m.formatExt("Image.webp") === "WEBP", "ext webp")
  }

  function testWatcher(w) {
    if (!w) return
    check("watcher", w._fileType("a.jpg") === "static", "jpg static")
    check("watcher", w._fileType("a.png") === "static", "png static")
    check("watcher", w._fileType("a.webp") === "static", "webp static")
    check("watcher", w._fileType("a.mp4") === "video", "mp4 video")
    check("watcher", w._fileType("a.mkv") === "video", "mkv video")
    check("watcher", w._fileType("a.gif") === "video", "gif video")
    check("watcher", w._fileType("a.JPG") === "static", "uppercase ext")
    check("watcher", w._fileType("a.txt") === "", "txt unknown")
    check("watcher", w._fileType("noext") === "", "no extension")
  }

  function testTagCloud(t) {
    if (!t) return
    check("tagcloud", t._stem("running") === "run", "running->run got " + t._stem("running"))
    check("tagcloud", t._stem("cats") === "cat", "cats->cat")
    check("tagcloud", t._stem("berries") === "berry", "berries->berry got " + t._stem("berries"))
    check("tagcloud", t._stem("leaves") === "leaf", "leaves->leaf got " + t._stem("leaves"))
    check("tagcloud", t._stem("walked") === "walk", "walked->walk")
    check("tagcloud", t._stem("stopped") === "stop", "stopped->stop got " + t._stem("stopped"))
    check("tagcloud", t._stem("faster") === "fast", "faster->fast")
    check("tagcloud", t._stem("quickly") === "quick", "quickly->quick")
    check("tagcloud", t._stem("go") === "go", "short word unchanged")
    check("tagcloud", t._stem("kiss") === "kiss", "ss not stripped got " + t._stem("kiss"))
    check("tagcloud", t._editDist("", "") === 0, "editDist empty")
    check("tagcloud", t._editDist("a", "") === 1, "editDist insert")
    check("tagcloud", t._editDist("cat", "cat") === 0, "editDist identical")
    check("tagcloud", t._editDist("cat", "bat") === 1, "editDist sub")
    check("tagcloud", t._editDist("kitten", "sitting") === 3, "editDist kitten/sitting got " + t._editDist("kitten", "sitting"))
    check("tagcloud", t._fuzzyMatch("mountain", "mount") === true, "fuzzy substring")
    check("tagcloud", t._fuzzyMatch("mountains", "mountain") === true, "fuzzy stem")
    check("tagcloud", t._fuzzyMatch("cat", "dog") === false, "fuzzy no-match")
  }

  function testWallhaven(wh) {
    if (!wh) return
    var u = wh._buildUrl()
    check("wallhaven", u.indexOf("https://wallhaven.cc/api/v1/search?") === 0, "base url")
    check("wallhaven", u.indexOf("categories=111") !== -1, "default categories")
    check("wallhaven", u.indexOf("purity=100") !== -1, "default purity")
    check("wallhaven", u.indexOf("topRange=1M") !== -1, "toplist adds topRange")
    check("wallhaven", u.indexOf("page=1") !== -1, "default page")
    wh.query = "cool cars"
    wh.selectedColor = 0
    wh.apiKey = "KEY123"
    var u2 = wh._buildUrl()
    check("wallhaven", u2.indexOf("q=cool%20cars") !== -1, "query encoded got " + u2)
    check("wallhaven", u2.indexOf("colors=cc0000") !== -1, "color 0 -> cc0000")
    check("wallhaven", u2.indexOf("apikey=KEY123") !== -1, "apikey present")
    wh.selectedColor = 8
    check("wallhaven", wh._buildUrl().indexOf("colors=333399") !== -1, "color 8 -> 333399")
    wh.sorting = "date_added"
    check("wallhaven", wh._buildUrl().indexOf("topRange=") === -1, "non-toplist omits topRange")
  }

  function testSteam(sw) {
    if (!sw) return
    sw.requiredType = ""
    sw.excludedTags = []
    sw.sorting = "new"
    var u = sw._buildUrl()
    check("steam", u.indexOf("https://api.steampowered.com/IPublishedFileService/QueryFiles/v1/?") === 0, "base url")
    check("steam", u.indexOf("query_type=1") !== -1, "new -> query_type 1 got " + u)
    sw.sorting = "toprated"
    check("steam", sw._buildUrl().indexOf("query_type=0") !== -1, "toprated -> 0")
    sw.sorting = "popular"
    check("steam", sw._buildUrl().indexOf("query_type=9") !== -1, "popular -> 9")
    sw.sorting = "trend"
    sw.query = ""
    check("steam", sw._buildUrl().indexOf("query_type=3") !== -1, "trend -> 3")
    check("steam", sw._buildUrl().indexOf("days=7") !== -1, "trend no-query adds days")
    sw.query = "forest"
    var uq = sw._buildUrl()
    check("steam", uq.indexOf("query_type=12") !== -1, "query overrides type to 12")
    check("steam", uq.indexOf("search_text=forest") !== -1, "search_text present")
    check("steam", uq.indexOf("days=") === -1, "query suppresses days")
    sw.apiKey = "K"
    check("steam", sw._buildUrl().indexOf("key=K") !== -1, "apikey present")
    sw.query = ""
    sw.requiredType = "Video"
    check("steam", sw._buildUrl().indexOf("requiredtags[0]=Video") !== -1, "requiredType tag")
  }

  function testTagPillFlow(tf) {
    if (!tf) return
    tf.retagging = false
    tf.tags = ["a", "b"]
    check("tagflow", eq(tf._displayed, ["a", "b"]), "non-retag change swaps immediately got " + JSON.stringify(tf._displayed))
    tf.retagging = true
    tf.tags = ["c", "d"]
    check("tagflow", tf.tagsExiting === true, "retag change starts exit transition")
    check("tagflow", eq(tf._displayed, ["a", "b"]), "retag keeps old tags until swap got " + JSON.stringify(tf._displayed))
    tf.tagsExiting = false
    tf.retagging = false
    tf.tags = ["x"]
    check("tagflow", eq(tf._displayed, ["x"]), "manual change after retag swaps immediately got " + JSON.stringify(tf._displayed))
  }

  function mkItem(o) {
    return {
      name: o.name !== undefined ? o.name : "",
      type: o.type !== undefined ? o.type : "static",
      thumb: o.thumb !== undefined ? o.thumb : (o.name || "x") + ".webp",
      path: o.path !== undefined ? o.path : "/tmp/wp/" + (o.name || "x"),
      weId: o.weId !== undefined ? o.weId : "",
      videoFile: o.videoFile !== undefined ? o.videoFile : "",
      mtime: o.mtime !== undefined ? o.mtime : 0,
      hue: o.hue !== undefined ? o.hue : 0,
      saturation: o.saturation !== undefined ? o.saturation : 0,
      richness: o.richness !== undefined ? o.richness : 0,
      applyCount: o.applyCount !== undefined ? o.applyCount : 0,
      placeholder: false
    }
  }

  function testSelector(svc, analysis) {
    if (!svc) return

    function namesOf() {
      var out = []
      for (var i = 0; i < svc.filteredModel.count; i++)
        out.push(svc.filteredModel.get(i).name)
      return out
    }
    function resetCache(mode) {
      svc.sortMode = mode
      svc._wallpaperData = []
      svc._wallpaperDataKeys = ({})
      svc.filteredModel.clear()
    }
    function cache(o) {
      svc._daemonConn.onItemCached(o)
    }
    function applyFilter(items) {
      svc._wallpaperData = items
      svc.updateFilteredModel(true)
      svc.commitFilteredModel()
      return namesOf()
    }

    resetCache("date")
    cache({ name: "a", thumb: "t", mtime: 100 })
    cache({ name: "b", thumb: "t", mtime: 300 })
    cache({ name: "c", thumb: "t", mtime: 200 })
    check("selector", eq(namesOf(), ["b", "c", "a"]), "incremental date order got " + JSON.stringify(namesOf()))

    resetCache("color")
    cache({ name: "unknown", thumb: "t", hue: 99, sat: 5 })
    cache({ name: "low", thumb: "t", hue: 10, sat: 5 })
    cache({ name: "mid", thumb: "t", hue: 50, sat: 5 })
    check("selector", eq(namesOf(), ["low", "mid", "unknown"]), "incremental color order unknown-hue-last")

    resetCache("date")
    cache({ name: "dup", thumb: "t", mtime: 1 })
    cache({ name: "dup", thumb: "t", mtime: 999 })
    check("selector", svc.filteredModel.count === 1, "dedup by key")

    resetCache("date")
    cache({ name: "x", thumb: "", mtime: 1 })
    cache({ name: "", thumb: "t", mtime: 1 })
    check("selector", svc.filteredModel.count === 0, "skip missing name/thumb")

    resetCache("date")
    cache({ name: "pic.webp", thumb: "t", mtime: 1, type: "static" })
    check("selector", svc.filteredModel.count === 1 && svc.filteredModel.get(0).path === "/tmp/wp/pic.webp", "static path built")

    svc.selectedTypeFilter = ""
    svc.selectedColorFilter = -1
    svc.favouriteFilterActive = false
    svc.selectedTags = []
    svc.tagsMatchAny = false
    svc.weatherFilterActive = false
    svc.tagsDb = ({})
    svc.weatherDb = ({})
    svc.favouritesDb = ({})

    svc.sortMode = "pop"
    var popOut = applyFilter([
      mkItem({ name: "lowpop", saturation: 10, richness: 60 }),
      mkItem({ name: "hipop", saturation: 90, richness: 0 })
    ])
    check("selector", eq(popOut, ["hipop", "lowpop"]), "pop sort got " + JSON.stringify(popOut))

    svc.sortMode = "richness"
    var richOut = applyFilter([
      mkItem({ name: "plain", richness: 2 }),
      mkItem({ name: "busy", richness: 9 }),
      mkItem({ name: "mid", richness: 5 })
    ])
    check("selector", eq(richOut, ["busy", "mid", "plain"]), "richness desc got " + JSON.stringify(richOut))

    svc.sortMode = "minimalist"
    var minOut = applyFilter([
      mkItem({ name: "plain", richness: 2 }),
      mkItem({ name: "busy", richness: 9 })
    ])
    check("selector", eq(minOut, ["plain", "busy"]), "minimalist asc")

    svc.sortMode = "applied"
    var appOut = applyFilter([
      mkItem({ name: "rare", applyCount: 1, mtime: 9 }),
      mkItem({ name: "fav", applyCount: 8, mtime: 1 })
    ])
    check("selector", eq(appOut, ["fav", "rare"]), "applied desc")

    svc.sortMode = "date"
    svc.selectedTypeFilter = "video"
    var typeOut = applyFilter([
      mkItem({ name: "img", type: "static", mtime: 2 }),
      mkItem({ name: "vid", type: "video", mtime: 1 })
    ])
    check("selector", eq(typeOut, ["vid"]), "type filter video-only got " + JSON.stringify(typeOut))
    svc.selectedTypeFilter = ""

    svc.selectedColorFilter = 4
    var colorOut = applyFilter([
      mkItem({ name: "green", hue: 4, mtime: 2 }),
      mkItem({ name: "red", hue: 0, mtime: 1 })
    ])
    check("selector", eq(colorOut, ["green"]), "color filter hue4 got " + JSON.stringify(colorOut))
    svc.selectedColorFilter = -1

    svc.sortMode = "date"
    var folderItems = [
      mkItem({ name: "root1.webp", mtime: 6 }),
      mkItem({ name: "effects/root1-invert.webp", mtime: 5 }),
      mkItem({ name: "nature/forest.webp", mtime: 4 }),
      mkItem({ name: "root2.webp", mtime: 3 }),
      mkItem({ name: "Zero Two/002 [Darling]", type: "we", weId: "zt", mtime: 2 }),
      mkItem({ name: "Winter / anime 4K", type: "video", videoFile: "/v/w.mp4", mtime: 1 })
    ]
    svc.selectedFolder = ""
    check("selector", eq(applyFilter(folderItems), ["root1.webp", "root2.webp", "Zero Two/002 [Darling]", "Winter / anime 4K"]), "folder default root-only; WE/video titles with slash stay in Main got " + JSON.stringify(namesOf()))
    check("selector", eq(svc.availableFolders, ["effects", "nature"]), "availableFolders only from static subfolders got " + JSON.stringify(svc.availableFolders))
    svc.selectedFolder = "effects"
    check("selector", eq(applyFilter(folderItems), ["effects/root1-invert.webp"]), "folder effects-only excludes WE/video got " + JSON.stringify(namesOf()))
    svc.selectedFolder = "*"
    check("selector", applyFilter(folderItems).length === 6, "folder all shows everything got " + JSON.stringify(namesOf()))
    svc.selectedFolder = ""

    svc.favouriteFilterActive = true
    svc.favouritesDb = ({ "favone": true })
    var favOut = applyFilter([
      mkItem({ name: "favone", mtime: 2 }),
      mkItem({ name: "plain", mtime: 1 })
    ])
    check("selector", eq(favOut, ["favone"]), "favourites filter got " + JSON.stringify(favOut))
    svc.favouriteFilterActive = false
    svc.favouritesDb = ({})

    svc.tagsDb = ({ "ka": ["sunset", "beach"], "kb": ["forest"] })
    var tagItems = [
      mkItem({ name: "a", thumb: "ka.webp", mtime: 2 }),
      mkItem({ name: "b", thumb: "kb.webp", mtime: 1 })
    ]
    svc.selectedTags = ["sunset"]
    check("selector", eq(applyFilter(tagItems), ["a"]), "tag single match got " + JSON.stringify(namesOf()))
    svc.selectedTags = ["sunset", "beach"]
    check("selector", eq(applyFilter(tagItems), ["a"]), "tag AND both present")
    svc.selectedTags = ["sunset", "forest"]
    check("selector", eq(applyFilter(tagItems), []), "tag AND no single item has both")
    svc.tagsMatchAny = true
    svc.selectedTags = ["sunset", "forest"]
    check("selector", eq(applyFilter(tagItems), ["a", "b"]), "tag OR matches either got " + JSON.stringify(namesOf()))
    svc.tagsMatchAny = false
    svc.selectedTags = ["-forest"]
    check("selector", eq(applyFilter(tagItems), ["a"]), "tag NOT excludes forest got " + JSON.stringify(namesOf()))
    svc.selectedTags = []
    svc.tagsDb = ({})

    svc.weatherFilterActive = true
    check("selector", svc.weatherFilterActive === false, "weatherFilterActive auto-disables when weather cannot be fetched")
    svc.weatherFilterActive = false
    svc.currentWeather = []
    svc.weatherDb = ({})

    svc.tagsDb = ({ "k1": ["sunset", "beach"], "k2": ["sunset", "forest"], "k3": ["beach"] })
    svc._rebuildPopularTags()
    var pt = {}
    for (var pi = 0; pi < svc.popularTags.length; pi++)
      pt[svc.popularTags[pi].tag] = svc.popularTags[pi].count
    check("selector", pt["sunset"] === 2 && pt["beach"] === 2 && pt["forest"] === 1, "popular tag counts got " + JSON.stringify(pt))
    check("selector", svc.popularTags.length > 0 && svc.popularTags[0].count === 2, "popular tags sorted desc")
    svc.tagsDb = ({})

    svc.filteredModel.clear()
    for (var ni = 0; ni < 5; ni++)
      svc.filteredModel.append({ name: "n" + ni, path: "p" + ni })
    check("selector", eq(svc._collectNeighbors("p2"), ["p3", "p1", "p4", "p0"]), "neighbors alternate fwd/back got " + JSON.stringify(svc._collectNeighbors("p2")))
    check("selector", eq(svc._collectNeighbors("nope"), []), "neighbors not-found empty")
    svc.filteredModel.clear()
    svc.filteredModel.append({ name: "only", path: "ponly" })
    check("selector", eq(svc._collectNeighbors("ponly"), []), "neighbors single item empty")

    resetCache("date")
    cache({ name: "keep", thumb: "t", mtime: 2 })
    cache({ name: "old", thumb: "t", mtime: 1 })
    svc._daemonConn.onFileRenamed("old", "renamed")
    check("selector", namesOf().indexOf("renamed") !== -1 && namesOf().indexOf("old") === -1, "rename updates model got " + JSON.stringify(namesOf()))
    check("selector", svc._wallpaperDataKeys["renamed"] === true && svc._wallpaperDataKeys["old"] === undefined, "rename updates keys")

    resetCache("date")
    cache({ name: "a", thumb: "t", mtime: 3 })
    cache({ name: "b", thumb: "t", mtime: 2 })
    cache({ name: "c", thumb: "t", mtime: 1 })
    svc._daemonConn.onFileRemoved("b", "static")
    check("selector", eq(namesOf(), ["a", "c"]), "remove drops item got " + JSON.stringify(namesOf()))
    check("selector", svc._wallpaperDataKeys["b"] === undefined, "remove clears key")

    resetCache("date")
    cache({ name: "p1", thumb: "t", mtime: 3 })
    cache({ name: "p2", thumb: "t", mtime: 2 })
    cache({ name: "keep", thumb: "t", mtime: 1 })
    svc._daemonConn.onFolderRemoved(["p1", "p2"])
    check("selector", eq(namesOf(), ["keep"]), "folder-remove batch got " + JSON.stringify(namesOf()))

    svc.favouritesDb = ({ "fav1": true, "we42": true })
    check("selector", svc.isFavourite("fav1", "") === true, "isFavourite by name")
    check("selector", svc.isFavourite("nope", "we42") === true, "isFavourite by weId")
    check("selector", svc.isFavourite("nope", "") === false, "isFavourite false")
    svc.favouritesDb = ({})

    svc.tagsDb = ({ "we99": ["alpha", "beta"] })
    check("selector", eq(svc.getWallpaperTags("name", "we99", "thumb"), ["alpha", "beta"]), "getWallpaperTags by weId got " + JSON.stringify(svc.getWallpaperTags("name", "we99", "thumb")))
    check("selector", eq(svc.getWallpaperTags("missing", "", "missing.webp"), []), "getWallpaperTags missing empty")
    svc.tagsDb = ({})

    resetCache("date")
    cache({ name: "photo.jpg", thumb: "t", mtime: 1, apply_count: 2 })
    svc._daemonConn.onWallpaperApplied("static", "photo.jpg", "/p", "", "")
    check("selector", svc._wallpaperData[0].applyCount === 3, "applyCount increments in data got " + svc._wallpaperData[0].applyCount)
    check("selector", svc.filteredModel.get(0).applyCount === 3, "applyCount increments in model")
    svc._daemonConn.onWallpaperApplied("static", "nomatch.jpg", "/p", "", "")
    check("selector", svc._wallpaperData[0].applyCount === 3, "applyCount unchanged for non-match")

    resetCache("color")
    cache({ name: "a", thumb: "ka.webp", hue: 0, sat: 50, mtime: 1 })
    svc.tagsDb = ({})
    if (analysis) analysis.running = true
    svc._analysisConn.onItemAnalyzed("ka", ["freshtag"], { hue: 0, saturation: 50 }, [])
    check("selector", eq(svc.tagsDb["ka"], ["freshtag"]), "onItemAnalyzed stores tags got " + JSON.stringify(svc.tagsDb["ka"]))
    check("selector", svc._analysisItemsDirty === true, "full-scan onItemAnalyzed batches (stays dirty)")
    svc._analysisConn.onAnalysisComplete()
    check("selector", svc._analysisItemsDirty === false, "onAnalysisComplete clears dirty")
    var hasFresh = false
    for (var ai = 0; ai < svc.popularTags.length; ai++)
      if (svc.popularTags[ai].tag === "freshtag") hasFresh = true
    check("selector", hasFresh, "onAnalysisComplete rebuilds popular tags with new tag")
    svc.tagsDb = ({})

    if (analysis) analysis.running = false
    svc._analysisItemsDirty = false
    svc._analysisConn.onItemAnalyzed("kb", ["instant"], { hue: 0, saturation: 0 }, [])
    check("selector", eq(svc.tagsDb["kb"], ["instant"]), "single retag stores tags got " + JSON.stringify(svc.tagsDb["kb"]))
    check("selector", svc._analysisItemsDirty === false, "single retag applies immediately (reassigns, no pending dirty)")
    svc.tagsDb = ({})
  }

  Component.onCompleted: {
    var qmlDir = Quickshell.env("SKWD_WALL_QML")
    if (!qmlDir) {
      console.error("QMLTEST ENV ERROR: SKWD_WALL_QML unset")
      root.resultCode = 2
      return
    }

    var S = Qt.createQmlObject(
      'import QtQuick\n' +
      'import "services"\n' +
      'QtObject {\n' +
      '  property var color: ColorMapping\n' +
      '  property var image: ImageService\n' +
      '  property var meta: FileMetadataService\n' +
      '  property var watch: WatcherService\n' +
      '  property var analysis: WallpaperAnalysisService\n' +
      '}',
      root, "file://" + qmlDir + "/__singletons.qml")

    function comp(rel, props) {
      var c = Qt.createComponent("file://" + qmlDir + "/" + rel)
      if (c.status !== Component.Ready) {
        root.fails.push("LOAD " + rel + ": " + c.errorString())
        return null
      }
      return c.createObject(root, props || {})
    }

    testColor(S ? S.color : null)
    testImage(S ? S.image : null)
    testMeta(S ? S.meta : null)
    testWatcher(S ? S.watch : null)
    testTagCloud(comp("wallpaper/TagCloud.qml", {}))
    testTagPillFlow(comp("wallpaper/TagPillFlow.qml", {}))
    testWallhaven(comp("wallpaper/WallhavenService.qml", { wallpaperDir: "/tmp/wp" }))
    testSteam(comp("wallpaper/SteamWorkshopService.qml", { weDir: "/tmp/we" }))
    testSelector(comp("wallpaper/WallpaperSelectorService.qml", {
      scriptsDir: "/tmp", homeDir: "/tmp", wallpaperDir: "/tmp/wp", videoDir: "/tmp/vid",
      cacheBaseDir: "/tmp/cache", weDir: "/tmp/we", weAssetsDir: "/tmp/wea", showing: false
    }), S ? S.analysis : null)

    if (root.fails.length > 0) {
      console.error("QMLTEST FAIL (" + root.fails.length + " of " + root.checks + " checks):")
      for (var i = 0; i < root.fails.length; i++)
        console.error("  - " + root.fails[i])
      root.resultCode = 1
    } else {
      console.warn("QMLTEST PASS (" + root.checks + " checks)")
      root.resultCode = 0
    }
  }
}
