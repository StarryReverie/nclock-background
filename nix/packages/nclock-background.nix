{
  lib,
  rustPlatform,

  autoPatchelfHook,
  makeWrapper,

  libGL,
  libgcc,
  wayland,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "nclock-background";
  version = "0.1.0";

  src = import ./source.nix { inherit lib; };

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  buildAndTestSubdir = [ "crates/nclock-background" ];

  nativeBuildInputs = [
    autoPatchelfHook
    makeWrapper
  ];

  buildInputs = [
    libgcc
  ];

  runtimeDependencies = [
    libGL
    wayland
  ];

  meta = {
    description = "Fancy dynamic night clock wallpaper engine for Wayland compositors";
    homepage = "https://github.com/starryreverie/nclock-background";
    mainProgram = "nclock-background";
    license = lib.licenses.gpl3Plus;
    maintainers = with lib.maintainers; [ starryreverie ];
    platforms = lib.platforms.linux;
  };
})
