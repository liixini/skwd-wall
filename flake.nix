{
  description = "Quickshell-based image, video & Wallpaper Image wallpaper selector with color sorting, Matugen integration, and more";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    quickshell.url = "github:quickshell-mirror/quickshell";
    skwd-daemon.url = "github:liixini/skwd-daemon";
  };

  outputs = { self, nixpkgs, quickshell, skwd-daemon, ... }:
    let
      forEachSystem = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
    in {
      packages = forEachSystem (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          qsPkgs = quickshell.inputs.nixpkgs.legacyPackages.${system};

          qtModules = with qsPkgs.qt6; [
            qtimageformats
            qtmultimedia
            qtsvg
            qt5compat
            qtwayland
          ];

          quickshellWithModules = quickshell.packages.${system}.default.withModules qtModules;

          qtPluginPath = pkgs.lib.makeSearchPath "lib/qt-6/plugins" qtModules;

          daemon = skwd-daemon.packages.${system}.default;

          runtimeDeps = with pkgs; [
            daemon
            matugen
            ffmpeg
            imagemagick
            inotify-tools
            curl
            file
            coreutils
            bash
            findutils
            xdg-utils
          ];

          daemonDeps = runtimeDeps ++ [ quickshellWithModules ];

          fonts = with pkgs; [
            nerd-fonts.symbols-only
            roboto
            roboto-mono
            material-design-icons
          ];
        in {
          default = pkgs.stdenv.mkDerivation {
            pname = "skwd-wall";
            version = "unstable";
            src = ./.;

            nativeBuildInputs = [ pkgs.makeWrapper ];

            installPhase = ''
              mkdir -p $out/share/skwd-wall
              cp -a shell.qml qml/ $out/share/skwd-wall/

              mkdir -p $out/share/skwd-wall/data
              cp -a data/matugen/ $out/share/skwd-wall/data/
              cp -a data/scripts/ $out/share/skwd-wall/data/
              install -Dm644 data/config.json.example $out/share/skwd-wall/data/config.json.example

              install -Dm644 data/skwd-wall.desktop $out/share/applications/skwd-wall.desktop

              makeWrapper ${quickshellWithModules}/bin/quickshell $out/bin/skwd-wall \
                --prefix PATH : ${pkgs.lib.makeBinPath runtimeDeps} \
                --prefix QT_PLUGIN_PATH : "${qtPluginPath}" \
                --set-default QSG_RHI_BACKEND vulkan \
                --add-flags "-p $out/share/skwd-wall/shell.qml"

              makeWrapper ${daemon}/bin/skwd $out/bin/skwd \
                --prefix PATH : ${pkgs.lib.makeBinPath daemonDeps} \
                --prefix QT_PLUGIN_PATH : "${qtPluginPath}" \
                --set-default QSG_RHI_BACKEND vulkan \
                --set SKWD_SHELL_QML "$out/share/skwd-wall/shell.qml" \
                --set SKWD_DATA_DIR "$out/share/skwd-wall/data" \
                --set SKWD_HOST_QML "${daemon}/share/skwd/skwd-daemon/host/shell.qml"

              makeWrapper ${daemon}/bin/skwd-daemon $out/bin/skwd-daemon \
                --prefix PATH : ${pkgs.lib.makeBinPath daemonDeps} \
                --prefix QT_PLUGIN_PATH : "${qtPluginPath}" \
                --set-default QSG_RHI_BACKEND vulkan \
                --set SKWD_SHELL_QML "$out/share/skwd-wall/shell.qml" \
                --set SKWD_DATA_DIR "$out/share/skwd-wall/data" \
                --set SKWD_HOST_QML "${daemon}/share/skwd/skwd-daemon/host/shell.qml"

              mkdir -p $out/lib/systemd/user
              substitute ${daemon}/lib/systemd/user/skwd-daemon.service \
                $out/lib/systemd/user/skwd-daemon.service \
                --replace-fail "${daemon}/bin/skwd-daemon" "$out/bin/skwd-daemon"

              install -Dm644 LICENSE $out/share/licenses/skwd-wall/LICENSE

              mkdir -p $out/share/fonts
              for font in ${pkgs.lib.concatMapStringsSep " " toString fonts}; do
                if [ -d "$font/share/fonts" ]; then
                  for f in $(find "$font/share/fonts" -type f); do
                    ln -sf "$f" "$out/share/fonts/$(basename $f)"
                  done
                fi
              done
            '';

            meta = {
              description = "Quickshell-based image, video & Wallpaper Image wallpaper selector with color sorting, Matugen integration, and more";
              homepage = "https://github.com/liixini/skwd-wall";
              license = pkgs.lib.licenses.mit;
              mainProgram = "skwd-wall";
            };
          };
        });

      nixosModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.programs.skwd-wall;
          skwd = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
        in {
          options.programs.skwd-wall.enable =
            lib.mkEnableOption "Skwd-wall (wallpaper selector + skwd-daemon user service)";

          config = lib.mkIf cfg.enable {
            environment.systemPackages = [ skwd ];
            systemd.packages = [ skwd ];
          };
        };
    };
}
