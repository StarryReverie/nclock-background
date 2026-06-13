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
        nclock-screensaver = pkgs.callPackage ../packages/nclock-screensaver.nix {
          nclock-background = config.packages.nclock-background;
        };
      };

      legacyPackages = config.packages;
    };
}
