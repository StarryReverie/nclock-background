{
  config,
  inputs,
  self,
  ...
}:
{
  flake.overlays = {
    default = self.overlays.nclock-background;
    nclock-background = import ../overlay.nix;
  };
}
