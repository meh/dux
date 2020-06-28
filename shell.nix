let
  pkgs = import ./nixpkgs.nix {};
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
      cargo
      dbus
      gcc
      pkg-config
      python3
      rustc
      xorg.libxcb
      xorg.xcbutilimage
      xorg.xcbutilwm
    ];
  }
