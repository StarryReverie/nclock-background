{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "nclock-background";
  version = "0.1.0";

  src = import ./source.nix { inherit lib; };

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  meta = {
    description = "Fancy dynamic night clock wallpaper engine for Wayland compositors";
    homepage = "https://github.com/starryreverie/nclock-background";
    mainProgram = "nclock-background";
    license = lib.licenses.gpl3Plus;
    maintainers = with lib.maintainers; [ starryreverie ];
    platforms = lib.platforms.linux;
  };
})
