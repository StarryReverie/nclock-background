final: prev: {
  nclock-background = prev.callPackage ./packages/nclock-background.nix { };
  nclock-screensaver = prev.callPackage ./packages/nclock-screensaver.nix { };
}
