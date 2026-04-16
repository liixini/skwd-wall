{
  description = "Quickshell-based image, video & Wallpaper Image wallpaper selector with color sorting, Matugen integration, and more";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    quickshell.url = "github:quickshell-mirror/quickshell";
    awww.url = "git+https://codeberg.org/LGFae/awww";
    skwd-daemon.url = "github:liixini/skwd-daemon";
  };

  outputs = { self, nixpkgs, quickshell, awww, skwd-daemon, ... }:
    let
      forEachSystem = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
    in {
      packages = forEachSystem (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          qsPkgs = quickshell.inputs.nixpkgs.legacyPackages.${system};

          quickshellWithModules = quickshell.packages.${system}.default.withModules (with qsPkgs.qt6; [
            qtmultimedia
            qtsvg
            qt5compat
            qtwayland
          ]);

          daemon = skwd-daemon.packages.${system}.default;

          runtimeDeps = with pkgs; [
            daemon
            matugen
            ffmpeg
            imagemagick
            inotify-tools
            sqlite
            curl
            mpvpaper
            jq
            awww.packages.${system}.awww
          ];

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
                --add-flags "-p $out/share/skwd-wall/shell.qml"

              makeWrapper ${daemon}/bin/skwd $out/bin/skwd \
                --prefix PATH : ${pkgs.lib.makeBinPath runtimeDeps}

              install -Dm644 ${daemon}/lib/systemd/user/skwd-daemon.service \
                $out/lib/systemd/user/skwd-daemon.service

              install -Dm644 LICENSE $out/share/licenses/skwd-wall/LICENSE

              # Symlink fonts into the package so they're available system-wide
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
    };
}
