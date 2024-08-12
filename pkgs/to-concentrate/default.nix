{ lib
, rustPlatform
}:

rustPlatform.buildRustPackage {
  pname = "to-concentrate";
  version = "0.1.0";
  src = ../../.;
  cargoLock.lockFile = ../../Cargo.lock;

  meta = {
    description = "A notifier daemon written in Rust which makes practical use of the tomato clock method";
    homepage = "";
    license = lib.licenses.gpl3Plus;
    maintainers = with lib.maintainers; [];
    mainProgram = "to-concentrate";
    platforms = lib.platforms.unix;
  };
}
