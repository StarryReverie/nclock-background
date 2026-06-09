{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "nclock-screensaver";
  version = "0.1.0";

  src = import ./source.nix { inherit lib; };

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  meta = {
    description = "Night clock screensaver";
    homepage = "https://github.com/starryreverie/nclock-screensaver";
    mainProgram = "nclock-screensaver";
    license = lib.licenses.gpl3Plus;
    maintainers = with lib.maintainers; [ starryreverie ];
    platforms = lib.platforms.linux;
  };
})
