{ self }:

{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.skwd-walld;
in
{
  options.services.skwd-walld = {
    enable = lib.mkEnableOption "skwd-wall control service and binaries";
    
    extraPackages = lib.mkOption {
      type = lib.types.listOf lib.types.package;
      default = [ ];
      description = "Optional Deck backends such as skwd-deck-steamworks.";
    };

    modelPackage = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.system}.skwd-lens-model;
      description = "Semantic model pack used by Skwd Lens. The default SigLIP 2 pack is installed with the suite.";
    };
  };

  config = lib.mkIf cfg.enable {

    home.packages = [
      self.packages.${pkgs.system}.default
      cfg.modelPackage
    ]
    ++ cfg.extraPackages;

    home.sessionVariables = {
      SKWD_LENS_HOME = "${cfg.modelPackage}/share/skwd-lens/models/semantic";
    };

    systemd.user.services.skwd-walld = {
      Unit = {
        Description = "skwd-wall control daemon";
        Conflicts = [ "skwd-daemon.service" ];
        After = [ "graphical-session.target" ];
        PartOf = [ "graphical-session.target" ];
      };

      Install = {
        WantedBy = [ "graphical-session.target" ];
      };

      Service = {
        ExecStart = "${self.packages.${pkgs.system}.deck}/bin/skwd-walld";
        Restart = "on-failure";
        Environment = [
          "SKWD_LENS_HOME=${cfg.modelPackage}/share/skwd-lens/models/semantic"
        ];
      };
    };

  };
}
