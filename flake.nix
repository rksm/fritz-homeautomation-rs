{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; };
      in
        {
          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rustc
              cargo
              clippy
              pkg-config
            ];

            buildInputs = with pkgs; [
              openssl
              clang
            ] ++ (if pkgs.stdenv.isDarwin then [ libiconv ] else [ ]);

            packages = with pkgs; [
              rust-analyzer
              cargo-outdated
              cargo-nextest
              (rustfmt.override { asNightly = true; })
            ];

            RUST_BACKTRACE = "1";
            RUST_LOG = "debug";
            LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          };
        }
    );
}
