{ lib
, rustPlatform
}:

rustPlatform.buildRustPackage {
  pname = "tomatify";
  version = "0.1.0";
  src = ../../.;
  cargoLock.lockFile = ../../Cargo.lock;

  meta = {
    description = "A Rust application";
    homepage = "";
    license = lib.licenses.gpl3Plus;
    maintainers = with lib.maintainers; [];
    mainProgram = "tomatify";
    platforms = lib.platforms.unix;
  };
}
