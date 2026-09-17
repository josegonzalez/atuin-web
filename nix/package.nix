{
  lib,
  rustPlatform,
}:

rustPlatform.buildRustPackage {
  pname = "atuin-web";
  version = (lib.importTOML ../Cargo.toml).package.version;

  src = lib.cleanSource ../.;

  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    description = "Web UI for atuin, to browse and search your shell history in a browser";
    homepage = "https://github.com/josegonzalez/atuin-web";
    license = lib.licenses.mit;
    mainProgram = "atuin-web";
    platforms = lib.platforms.unix;
  };
}
