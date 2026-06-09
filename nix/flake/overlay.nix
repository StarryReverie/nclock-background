{
  config,
  inputs,
  self,
  ...
}:
{
  flake.overlays = {
    default = self.overlays.nclock-screensaver;
    nclock-screensaver = import ../overlay.nix;
  };
}
