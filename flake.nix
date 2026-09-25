{
  description = "Skwd release binaries on NixOS";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs, home-manager }:
    let
      release = builtins.fromJSON (builtins.readFile ./nix/release.json);
      forAllSystems = nixpkgs.lib.genAttrs [ "x86_64-linux" ];
    in
    {
      packages = forAllSystems (
        system:
        import ./nix/binary-packages.nix {
          pkgs = import nixpkgs { inherit system; };
          inherit release;
        }
      );

      nixosModules.default = import ./nix/nixos.nix { inherit self; };

      homeModules.default = import ./nix/hm-module.nix { inherit self; };

      checks = forAllSystems (
        system:
        import ./nix/checks.nix {
          pkgs = import nixpkgs { inherit system; };
          packages = self.packages.${system};
          nixosModule = self.nixosModules.default;
          homeManagerModule = self.homeModules.default;
          inherit home-manager;
          inherit release;
        }
      );
    };
}
