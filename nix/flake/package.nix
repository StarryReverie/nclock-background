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
        default = config.packages.nclock-screensaver;
        nclock-screensaver = pkgs.callPackage ../package.nix { };
      };

      legacyPackages = config.packages;
    };
}
