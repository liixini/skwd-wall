{ pkgs, release }:
let
  graphicsRuntime = with pkgs; [
    libglvnd
    libxkbcommon
    vulkan-loader
    wayland
  ];
  names = {
    wall = "skwd-wall-v2";
    deck = "skwd-deck";
    paper = "skwd-paper";
    lens = "skwd-lens";
    model = "skwd-lens-model";
    plasma = "skwd-paper-plasma";
  };
  components = pkgs.lib.mapAttrs (
    component: source:
    pkgs.stdenvNoCC.mkDerivation {
      pname = names.${component};
      version = source.version;
      src = pkgs.fetchurl { inherit (source) url hash; };
      nativeBuildInputs = with pkgs; [
        autoPatchelfHook
        zstd
      ];
      buildInputs =
        with pkgs;
        [
          alsa-lib
          dav1d
          libdrm
          libpulseaudio
          libva-minimal
          libyuv
          shaderc
          stdenv.cc.cc.lib
          zlib
        ]
        ++ graphicsRuntime
        ++ pkgs.lib.optionals (component == "plasma") [
          pkgs.qt6.qtbase
          pkgs.qt6.qtdeclarative
        ]
        ++ pkgs.lib.optional (component == "deck") components.paper;
      runtimeDependencies = pkgs.lib.optionals (builtins.elem component [
        "wall"
        "deck"
        "paper"
      ]) graphicsRuntime;
      preFixup = ''
        while IFS= read -r -d "" file; do
          if isELF "$file"; then
            patchelf --remove-rpath "$file"
          fi
        done < <(find "$out" -type f -print0)
      ''
      + pkgs.lib.optionalString (component == "deck") ''
        addAutoPatchelfSearchPath ${components.paper}/lib/skwd-paper
      '';
      doInstallCheck = builtins.hasAttr component executables;
      installCheckPhase = ''
        runHook preInstallCheck
        ${pkgs.lib.optionalString (builtins.hasAttr component executables) ''"$out/bin/${executables.${component}}" --version''}
        runHook postInstallCheck
      '';
      dontConfigure = true;
      dontBuild = true;
      dontStrip = true;
      dontMoveSystemdUserUnits = true;
      dontWrapQtApps = true;
      propagatedBuildInputs =
        pkgs.lib.optional (component == "plasma") components.paper
        ++ pkgs.lib.optional (component == "lens") components.model;
      unpackPhase = ''
        export HOME="$TMPDIR/skwd-home"
        mkdir -p "$HOME"
        runHook preUnpack
        mkdir package
        tar -xf "$src" -C package
        runHook postUnpack
      '';
      installPhase = ''
        runHook preInstall
        mkdir -p "$out"
        cp -a package/usr/. "$out/"
        chmod -R u+w "$out"
        ${pkgs.lib.optionalString (component == "plasma") ''
          if [ -d "$out/lib/qt6/qml" ]; then
            mkdir -p "$out/${pkgs.qt6.qtbase.qtQmlPrefix}"
            cp -a "$out/lib/qt6/qml/." "$out/${pkgs.qt6.qtbase.qtQmlPrefix}/"
            rm -r "$out/lib/qt6"
          fi
        ''}
        runHook postInstall
      '';
      meta.platforms = [ "x86_64-linux" ];
    }
  ) release.binaries;
  executables = {
    wall = "skwd-wall-v2";
    deck = "skwd-walld";
    paper = "skwd-paper-v2";
    lens = "skwd-lens";
  };
in
components
// {
  skwd-wall-v2 = components.wall;
  skwd-deck = components.deck;
  skwd-paper = components.paper;
  skwd-lens = components.lens;
  default = pkgs.symlinkJoin {
    name = "skwd-suite-binary-${release.version}";
    paths = with components; [
      wall
      deck
      paper
      lens
      model
    ];
    nativeBuildInputs = [ pkgs.makeWrapper ];
    postBuild = ''
      for program in skwd-wall-v2 skwd-walld skwd-wall-scan; do
        wrapProgram "$out/bin/$program" \
          --set-default SKWD_LENS_HOME ${components.model}/share/skwd-lens/models/semantic \
          --prefix PATH : ${
            pkgs.lib.makeBinPath [
              components.deck
              components.paper
              components.lens
              pkgs.bash
              pkgs.coreutils
              pkgs.util-linux
            ]
          }
      done
      rm "$out/share/systemd/user/skwd-walld.service"
      substitute ${components.deck}/share/systemd/user/skwd-walld.service \
        "$out/share/systemd/user/skwd-walld.service" \
        --replace-fail /usr/bin/skwd-walld "$out/bin/skwd-walld"
    '';
  };
}
// pkgs.lib.optionalAttrs release.hasModel {
  skwd-lens-model = components.model;
}
// pkgs.lib.optionalAttrs release.hasPlasma {
  skwd-paper-plasma = components.plasma;
}
