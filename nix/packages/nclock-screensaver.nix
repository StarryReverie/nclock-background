{
  lib,
  rustPlatform,

  makeWrapper,

  nclock-background,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "nclock-screensaver";
  version = "0.1.0";

  src = import ./source.nix { inherit lib; };

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  buildAndTestSubdir = [ "crates/nclock-screensaver" ];

  nativeBuildInputs = [
    makeWrapper
  ];

  postInstall = ''
    wrapProgram $out/bin/nclock-screensaver \
      --prefix PATH : ${lib.makeBinPath [ nclock-background ]}
  '';

  meta = {
    description = "Screensaver adapter and management process of night clock wallpaper engine";
    homepage = "https://github.com/starryreverie/nclock-background";
    mainProgram = "nclock-screensaver";
    license = lib.licenses.gpl3Plus;
    maintainers = with lib.maintainers; [ starryreverie ];
    platforms = lib.platforms.linux;
  };
})
