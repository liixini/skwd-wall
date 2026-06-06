import QtQuick
import Quickshell

ShellRoot {
  id: root
  property int resultCode: -1
  property var fails: []
  property var svc: null
  property var dc: null
  property int phase: 0
  property int ticks: 0
  property int applyTick: 0
  property var lastOutputs: ({})
  property string alphaPath: ""
  property string bravoPath: ""
  property string clipPath: ""
  property string weId: ""

  Timer {
    interval: 1
    repeat: false
    running: root.resultCode >= 0
    onTriggered: Qt.exit(root.resultCode)
  }

  function fail(m) {
    root.fails.push(m)
  }

  function finish() {
    if (root.fails.length > 0) {
      console.error("E2E FAIL (" + root.fails.length + "):")
      for (var i = 0; i < root.fails.length; i++)
        console.error("  - " + root.fails[i])
      root.resultCode = 1
    } else {
      console.warn("E2E PASS")
      root.resultCode = 0
    }
  }

  function queryOutputs(cb) {
    root.dc.outputs(function (result, error) {
      cb((result && result.outputs) ? result.outputs : ({}))
    })
  }

  function findByType(t) {
    for (var i = 0; i < root.svc.filteredModel.count; i++) {
      var it = root.svc.filteredModel.get(i)
      if (it.type === t)
        return it
    }
    return null
  }

  function findByName(n) {
    for (var i = 0; i < root.svc.filteredModel.count; i++) {
      var it = root.svc.filteredModel.get(i)
      if (it.name === n)
        return it
    }
    return null
  }

  Timer {
    id: poll
    interval: 300
    repeat: true
    running: root.svc !== null
    onTriggered: {
      root.ticks++
      if (root.ticks > 300) {
        root.fail("timeout at phase " + root.phase + " outputs=" + JSON.stringify(root.lastOutputs))
        poll.stop()
        root.finish()
        return
      }
      root.step()
    }
  }

  function step() {
    var svc = root.svc

    if (root.phase === 0) {
      if (svc._wallpaperData.length >= 5) {
        svc.commitFilteredModel()
        var s = findByType("static")
        var v = findByType("video")
        var w = findByType("we")
        if (!s) root.fail("no static wallpaper loaded")
        if (!v) root.fail("no video wallpaper loaded")
        if (!w) root.fail("no WE wallpaper loaded")
        if (s && v && w) {
          var a = findByName("alpha.png")
          var b = findByName("bravo.png")
          var c = findByName("clip.mp4")
          root.alphaPath = a ? a.path : ""
          root.bravoPath = b ? b.path : ""
          root.clipPath = c ? c.path : ""
          root.weId = w.weId
        }
        root.phase = 1
      } else {
        svc.refreshFromDb()
      }
      return
    }

    if (root.phase === 1) {
      svc.sortMode = "date"
      svc.updateFilteredModel(true)
      svc.commitFilteredModel()
      var mono = true
      for (var i = 0; i + 1 < svc.filteredModel.count; i++)
        if (svc.filteredModel.get(i).mtime < svc.filteredModel.get(i + 1).mtime)
          mono = false
      if (!mono)
        root.fail("date sort not monotonic over real daemon data")
      root.phase = 2
      return
    }

    if (root.phase === 2) {
      if (!root.alphaPath || !root.clipPath || !root.weId) {
        root.fail("missing seeded paths: alpha=" + root.alphaPath + " clip=" + root.clipPath + " weId=" + root.weId)
        root.phase = 6
        return
      }
      svc.applyStatic(root.alphaPath, ["DP-1"])
      svc.applyVideo(root.clipPath, ["DP-2"], ({}), ({}))
      svc.applyWE(root.weId, ["DP-3"], ({}), ({}))
      root.applyTick = root.ticks
      root.phase = 3
      return
    }

    if (root.phase === 3) {
      if (root.phase !== 3)
        return
      queryOutputs(function (o) {
        root.lastOutputs = o
        var dp1 = o["DP-1"], dp2 = o["DP-2"], dp3 = o["DP-3"]
        var ok1 = dp1 && dp1.type === "static" && ("" + dp1.path).indexOf("alpha.png") !== -1
        var ok2 = dp2 && dp2.type === "video" && ("" + dp2.path).indexOf("clip.mp4") !== -1
        var ok3 = dp3 && dp3.type === "we"
        if (ok1 && ok2 && ok3 && root.phase === 3)
          root.phase = 4
      })
      if (root.ticks - root.applyTick > 120 && root.phase === 3) {
        if (!(root.lastOutputs["DP-1"] && root.lastOutputs["DP-1"].type === "static"))
          root.fail("static not applied to DP-1; outputs=" + JSON.stringify(root.lastOutputs))
        if (!(root.lastOutputs["DP-2"] && root.lastOutputs["DP-2"].type === "video"))
          root.fail("video not applied to DP-2; outputs=" + JSON.stringify(root.lastOutputs))
        if (!(root.lastOutputs["DP-3"] && root.lastOutputs["DP-3"].type === "we"))
          root.fail("WE not applied to DP-3; outputs=" + JSON.stringify(root.lastOutputs))
        root.phase = 6
      }
      return
    }

    if (root.phase === 4) {
      svc.applyStatic(root.bravoPath, ["DP-1"])
      root.applyTick = root.ticks
      root.phase = 5
      return
    }

    if (root.phase === 5) {
      queryOutputs(function (o) {
        root.lastOutputs = o
        var dp1 = o["DP-1"]
        if (dp1 && dp1.type === "static" && ("" + dp1.path).indexOf("bravo.png") !== -1 && root.phase === 5)
          root.phase = 6
      })
      if (root.ticks - root.applyTick > 60 && root.phase === 5) {
        root.fail("sequence switch to bravo not reflected on DP-1; outputs=" + JSON.stringify(root.lastOutputs))
        root.phase = 6
      }
      return
    }

    if (root.phase === 6) {
      poll.stop()
      root.finish()
    }
  }

  Component.onCompleted: {
    var qmlDir = Quickshell.env("SKWD_WALL_QML")
    var home = Quickshell.env("HOME")
    root.dc = Qt.createQmlObject(
      'import QtQuick\nimport "services"\nQtObject { property var c: DaemonClient }',
      root, "file://" + qmlDir + "/__dc.qml").c
    var comp = Qt.createComponent("file://" + qmlDir + "/wallpaper/WallpaperSelectorService.qml")
    if (comp.status !== Component.Ready) {
      root.fail("load: " + comp.errorString())
      root.finish()
      return
    }
    root.svc = comp.createObject(root, {
      scriptsDir: "/tmp",
      homeDir: home,
      wallpaperDir: home + "/Pictures/Wallpapers",
      videoDir: home + "/Pictures/Wallpapers",
      cacheBaseDir: Quickshell.env("XDG_CACHE_HOME") + "/skwd",
      weDir: home + "/we",
      weAssetsDir: "/tmp/wea",
      showing: false
    })
  }
}
