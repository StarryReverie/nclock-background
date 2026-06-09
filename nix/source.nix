{ lib }:
lib.fileset.toSource {
  root = ../.;
  fileset = lib.fileset.unions [
    ../Cargo.toml
    ../Cargo.lock
    ../src
  ];
}
