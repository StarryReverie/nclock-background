{
  config,
  inputs,
  self,
  ...
}:
{
  perSystem =
    { system, pkgsDev, ... }:
    {
      _module.args.pkgsDev = (import inputs.nixpkgs) {
        inherit system;
        overlays = [ inputs.rust-overlay.overlays.default ];
      };

      devShells.default = pkgsDev.mkShellNoCC {
        packages = [
          (pkgsDev.rust-bin.fromRustupToolchainFile ./../../../rust-toolchain.toml)

          pkgsDev.nixfmt
          pkgsDev.nixfmt-tree

          pkgsDev.libGL
          pkgsDev.libgcc
          pkgsDev.pkg-config
          pkgsDev.wayland
        ];

        shellHook = ''
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
            pkgsDev.lib.makeLibraryPath [
              pkgsDev.libGL
              pkgsDev.wayland
            ]
          }"
        '';
      };
    };
}
