{
  description = "skwd-wall";

  inputs.release.url = "github:liixini/skwd-wall/nix";

  outputs = { release, ... }: {
    packages = release.packages;
    checks = release.checks;
    nixosModules = release.nixosModules;
  };
}
