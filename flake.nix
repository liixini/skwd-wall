{
  description = "A wallpaper manager for quickshell";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    quickshell.url = "github:quickshell-mirror/quickshell";
    awww.url = "git+https://codeberg.org/LGFae/awww";
  };

  outputs = { self, nixpkgs, quickshell, awww, ... }:
    let
      foreachSystem = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ];
    in {
      packages = foreachSystem (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          qsPkgs = quickshell.inputs.nixpkgs.legacyPackages.${system};
          
          quickshellWithModules = quickshell.packages.${system}.default.withModules (with qsPkgs.qt6; [
            qtmultimedia
            qtsvg
            qt5compat
            qtwayland
          ]);

          runtimeDeps = with pkgs; [
            matugen 
            ffmpeg 
            imagemagick 
            inotify-tools 
            sqlite 
            curl
            file
            awww.packages.${system}.awww
          ];
        in {
          default = pkgs.stdenv.mkDerivation {
            pname = "skwd-wall";
            version = "unstable";
            src = ./.;

            nativeBuildInputs = [ pkgs.makeWrapper ];

            installPhase = ''
              mkdir -p $out/share/skwd-wall
              cp -r . $out/share/skwd-wall

              makeWrapper ${quickshellWithModules}/bin/quickshell $out/bin/skwd-wall-daemon \
                --prefix PATH : ${pkgs.lib.makeBinPath runtimeDeps} \
                --add-flags "-p $out/share/skwd-wall/daemon.qml"

              makeWrapper ${quickshellWithModules}/bin/quickshell $out/bin/skwd-wall-toggle \
                --add-flags "ipc -p $out/share/skwd-wall/daemon.qml call wallpaper toggle"
            '';
          };
        });
    };
}
