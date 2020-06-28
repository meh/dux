let
  # nixos-19.09 on 2020-03-03
  rev = "84f47bfe9ae892042fcb04f319ffe208cd0dbfd9";
  nixpkgs = builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/${rev}.tar.gz";
    sha256 = "0rh1kcasz78f2bbnd816705x027xkyrlm314x4jg8hjdhzyaky8f";
  };
in
  import nixpkgs
