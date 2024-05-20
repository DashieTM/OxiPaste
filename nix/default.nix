{ rustPlatform
, rust-bin
, pkg-config
, wrapGAppsHook4
, gtk4
, gtk4-layer-shell
, libadwaita
, dbus
, lib
, lockFile
, ...
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.package.version;

  src = ../.;

  buildInputs = [
    pkg-config
    gtk4
    gtk4-layer-shell
    libadwaita
    dbus
  ];

  cargoLock = {
    inherit lockFile;
  };

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook4
    (rust-bin.selectLatestNightlyWith
      (toolchain: toolchain.default))
  ];

  copyLibs = true;

  meta = with lib; {
    description = "A simple clipboard manager written in Rust and gtk4.";
    homepage = "https://github.com/DashieTM/OxiPaste";
    changelog = "https://github.com/DashieTM/OxiPaste/releases/tag/${version}";
    license = licenses.gpl3;
    maintainers = with maintainers; [ DashieTM ];
    mainProgram = "oxipaste";
  };
}
