{
  config,
  inputs,
  self,
  ...
}:
{
  perSystem =
    {
      config,
      system,
      pkgs,
      ...
    }:
    {
      packages = {
        default = config.packages.nclock-background;
        nclock-background = pkgs.callPackage ../package.nix { };
      };

      legacyPackages = config.packages;
    };
}
