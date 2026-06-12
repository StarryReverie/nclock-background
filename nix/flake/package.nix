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
        nclock-background = pkgs.callPackage ../packages/nclock-background.nix { };
      };

      legacyPackages = config.packages;
    };
}
