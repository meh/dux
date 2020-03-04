let
  pkgs = import ./nixpkgs.nix {};
in
  pkgs.mkShell {
    buildInputs = with pkgs; [
      cargo
      dbus
      gcc
      pkg-config
      rustc
      xorg.libxcb
      xorg.xcbutilimage
      xorg.xcbutilwm
    ];
  }
