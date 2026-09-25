{
  pkgs,
  packages,
  nixosModule,
  release,
  homeManagerModule,
  home-manager,
}:
{
  profile-install = pkgs.testers.runNixOSTest {
    name = "skwd-profile-install";
    nodes.machine = { ... }: {
      users.users.alice = {
        isNormalUser = true;
        uid = 1000;
      };
      nix.settings.experimental-features = [ ];
      virtualisation.additionalPaths = [ packages.default ];
      virtualisation.memorySize = 2048;
      system.stateVersion = "25.05";
    };
    testScript = ''
      import shlex

      def user(command):
          return "su - alice -c " + shlex.quote("export XDG_RUNTIME_DIR=/run/user/1000; " + command)

      start_all()
      machine.wait_for_unit("multi-user.target")
      machine.succeed("loginctl enable-linger alice")
      machine.wait_for_unit("user@1000.service")
      machine.fail(user("command -v skwd-wall-v2"))
      machine.fail(user("nix profile install ${packages.default}"))
      machine.succeed(user("nix --extra-experimental-features 'nix-command flakes' profile install ${packages.default}"))
      machine.succeed(user("skwd-wall-v2 --version; skwd-helm --version; skwd-lens --version"))
      ${pkgs.lib.optionalString release.hasModel ''
        machine.succeed(user("test -s ~/.nix-profile/share/skwd-lens/models/semantic/semantic-pack.json"))
        machine.succeed(user("test -f ~/.nix-profile/share/skwd-lens/models/semantic/runtime/libonnxruntime.so.1.27.0"))
      ''}
      machine.succeed(user("test -f ~/.nix-profile/share/applications/skwd-wall-v2.desktop"))
      machine.succeed(user("systemctl --user daemon-reload"))
      machine.fail(user("systemctl --user is-enabled skwd-walld"))
      machine.succeed(user("systemctl --user enable --now skwd-walld"))
      machine.sleep(3)
      machine.succeed(user("systemctl --user is-active skwd-walld"))
      pid = machine.succeed(user("systemctl --user show skwd-walld -p MainPID --value")).strip()
      assert pid != "0"
      assert "skwd-walld" in machine.succeed("readlink /proc/" + pid + "/exe")
      ${pkgs.lib.optionalString release.hasModel ''
        machine.succeed("tr '\\0' '\\n' < /proc/" + pid + "/environ | grep '^SKWD_LENS_HOME=/nix/store/'")
      ''}
      target = machine.succeed(user("readlink ~/.config/systemd/user/default.target.wants/skwd-walld.service")).strip()
      assert target == "/home/alice/.nix-profile/share/systemd/user/skwd-walld.service", target
      machine.succeed(user("systemctl --user disable --now skwd-walld"))
      machine.fail(user("systemctl --user is-active skwd-walld"))
      machine.fail(user("systemctl --user is-enabled skwd-walld"))
    '';
  };

  release-install = pkgs.testers.runNixOSTest {
    name = "skwd-release-install";
    nodes.machine = { lib, ... }: {
      imports = [ nixosModule ];
      services.skwd-deck.enable = true;
      users.users.alice = {
        isNormalUser = true;
        uid = 1000;
        extraGroups = [
          "video"
          "render"
        ];
      };
      services.getty.autologinUser = "alice";
      programs.sway.enable = true;
      programs.bash.loginShellInit = ''
        if [ "$(tty)" = /dev/tty1 ]; then
          exec sway
        fi
      '';
      environment.variables.WLR_RENDERER = "pixman";
      environment.systemPackages = [
        pkgs.grim
        pkgs.jq
      ]
      ++ lib.optional release.hasPlasma packages.skwd-paper-plasma;
      virtualisation.memorySize = 4096;
      virtualisation.cores = 2;
      virtualisation.qemu.options = [ "-vga none -device virtio-gpu-pci" ];
      system.stateVersion = "25.05";
    };
    testScript = ''
      import base64
      import json
      import shlex
      import struct
      import zlib

      start_all()
      machine.wait_for_unit("multi-user.target")
      machine.succeed("${packages.wall}/bin/skwd-wall-v2 --version")
      machine.succeed("${packages.deck}/bin/skwd-walld --version")
      machine.succeed("${packages.deck}/bin/skwd-helm --version")
      machine.succeed("${packages.lens}/bin/skwd-lens --version")
      machine.succeed("test -x ${packages.paper}/bin/skwd-paper-v2")
      machine.succeed("grep -F ${packages.deck}/bin/skwd-walld /etc/systemd/user/skwd-walld.service")
      machine.succeed("grep -F ${packages.paper}/bin /etc/systemd/user/skwd-walld.service")
      machine.succeed("grep -F ${packages.lens}/bin /etc/systemd/user/skwd-walld.service")
      ${pkgs.lib.optionalString release.hasModel ''
        machine.succeed("test -d ${packages.skwd-lens-model}/share/skwd-lens/models/semantic")
        machine.succeed("grep -F ${packages.skwd-lens-model}/share/skwd-lens/models/semantic /etc/systemd/user/skwd-walld.service")
        def chunk(kind, data):
            return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", zlib.crc32(kind + data))

        entries = []
        for key, color in [("red", bytes([255, 0, 0])), ("blue", bytes([0, 0, 255]))]:
            png = bytes([137, 80, 78, 71, 13, 10, 26, 10])
            png += chunk(b"IHDR", struct.pack(">IIBBBBB", 224, 224, 8, 2, 0, 0, 0))
            png += chunk(b"IDAT", zlib.compress((bytes([0]) + color * 224) * 224))
            png += chunk(b"IEND", b"")
            path = "/tmp/" + key + ".png"
            machine.succeed("echo " + base64.b64encode(png).decode() + " | base64 -d > " + path)
            entries.append({"key": key, "path": path, "fingerprint": 1})

        model = "${packages.skwd-lens-model}/share/skwd-lens/models/semantic"
        lens = "${packages.lens}/bin/skwd-lens --manifest " + model + "/semantic-pack.json"
        lens += " --runtime " + model + "/runtime/libonnxruntime.so.1.27.0 --index /tmp/lens.sidx --threads 2"
        catalog = shlex.quote(json.dumps({"fingerprint": 1, "entries": entries}))
        machine.succeed("echo " + catalog + " | " + lens + " --build-index")
        for color in ["red", "blue"]:
            response = machine.succeed(lens + " --query " + shlex.quote("a solid " + color + " background"))
            result = json.loads(response)
            assert result.get("error") is None, result
            assert result["matches"][0]["key"] == color, result
        machine.succeed("test -s /tmp/lens.sidx")
        def user(command):
            return "su - alice -c " + shlex.quote("export XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-1; " + command)

        machine.wait_for_file("/run/user/1000/wayland-1")
        machine.wait_until_succeeds(user("systemctl --user is-active skwd-walld"))
        machine.succeed(user("mkdir -p ~/Pictures/Wallpapers; cp /tmp/red.png /tmp/blue.png ~/Pictures/Wallpapers/"))
        machine.wait_until_succeeds(user("test -s ~/.cache/skwd-wall-v2/semantic/index-siglip2.sidx"))
        machine.succeed(user("systemd-run --user --unit=skwd-picker-test --collect env -u SKWD_LENS_HOME /run/current-system/sw/bin/skwd-wall-v2"))
        machine.wait_until_succeeds(user("skwd-helm ui state | jq -e '.shown and .viewport.w > 0'"))
        machine.succeed(user("skwd-helm ui search 'a solid blue background'"))
        machine.wait_until_succeeds(user("skwd-helm ui state | jq -e '.semantic.resolved and .semantic.error == null and .selection == \"static:blue.png\"'"))
        machine.sleep(1)
        machine.screenshot("describe-search")
        machine.succeed(user("skwd-helm apply ~/Pictures/Wallpapers/blue.png"))
        machine.wait_until_succeeds(user("skwd-helm current --json | jq -e '.outputs | any(.path | endswith(\"/blue.png\"))'"))
        machine.succeed(user("skwd-helm ui hide"))
        machine.wait_until_fails(user("systemctl --user is-active skwd-picker-test"))
        machine.sleep(1)
        machine.screenshot("wallpaper")
      ''}
      ${pkgs.lib.optionalString release.hasPlasma ''
        machine.succeed("test -f ${packages.skwd-paper-plasma}/share/plasma/wallpapers/org.skwd.wall.plasma/metadata.json")
        machine.succeed("test -f ${packages.skwd-paper-plasma}/${pkgs.qt6.qtbase.qtQmlPrefix}/org/skwd/wallpaper/qmldir")
        probe = "import QtQuick; import QtQuick.Window; import org.skwd.wallpaper 1.0; Window { width: 64; height: 64; visible: true; SkwdVideoItem {} Timer { interval: 100; running: true; onTriggered: Qt.quit() } }"
        machine.succeed("echo " + shlex.quote(probe) + " > /tmp/plasma-probe.qml")
        machine.succeed("QT_QPA_PLATFORM=offscreen ${pkgs.qt6.qtdeclarative}/bin/qml -I ${packages.skwd-paper-plasma}/${pkgs.qt6.qtbase.qtQmlPrefix} -I ${pkgs.qt6.qtdeclarative}/${pkgs.qt6.qtbase.qtQmlPrefix} /tmp/plasma-probe.qml")
      ''}
    '';
  };

  home-manager-install = pkgs.testers.runNixOSTest {
    name = "skwd-home-manager-install";
    nodes.machine = { ... }: {
      imports = [
        home-manager.nixosModules.home-manager
      ];

      users.users.alice = {
        isNormalUser = true;
        uid = 1000;
        extraGroups = [
          "video"
          "render"
        ];
      };

      services.getty.autologinUser = "alice";
      programs.sway.enable = true;
      programs.bash.loginShellInit = ''
        if [ "$(tty)" = /dev/tty1 ]; then
          exec sway
        fi
      '';

      environment.variables.WLR_RENDERER = "pixman";

      home-manager.useGlobalPkgs = true;
      home-manager.useUserPackages = true;
      home-manager.users.alice = {
        imports = [ homeManagerModule ];
        services.skwd-walld.enable = true;
        home.stateVersion = "25.05";
      };

      virtualisation.memorySize = 2048;
      virtualisation.qemu.options = [ "-vga none -device virtio-gpu-pci" ];
      system.stateVersion = "25.05";
    };

    testScript = ''
      import shlex

      def user(command):
          return "su - alice -c " + shlex.quote("export XDG_RUNTIME_DIR=/run/user/1000; export WAYLAND_DISPLAY=wayland-1; " + command)

      start_all()
      machine.wait_for_unit("multi-user.target")
      machine.succeed("loginctl enable-linger alice")
      machine.wait_for_unit("user@1000.service")
      machine.wait_for_file("/run/user/1000/wayland-1")

      machine.succeed(user("skwd-wall-v2 --version; skwd-helm --version; skwd-lens --version"))
      machine.succeed(user("systemctl --user daemon-reload"))
      machine.succeed(user("systemctl --user is-enabled skwd-walld"))
      machine.succeed(user("systemctl --user start skwd-walld"))
      machine.wait_until_succeeds(user("systemctl --user is-active skwd-walld"))
      
      pid = machine.succeed(user("systemctl --user show skwd-walld -p MainPID --value")).strip()
      assert pid != "0"
      assert "skwd-walld" in machine.succeed("readlink /proc/" + pid + "/exe")
      ${pkgs.lib.optionalString release.hasModel ''
        machine.succeed("tr '\\0' '\\n' < /proc/" + pid + "/environ | grep '^SKWD_LENS_HOME=/nix/store/'")
      ''}
    '';
  };
  
}
