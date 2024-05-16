{
  description = "the auth package";

  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        name = "ec-shell";
        nativeBuildInputs = with pkgs; [
          pkg-config
          postgresql_14
          openssl.dev
        ];
        buildInputs = with pkgs; [
          diesel-cli

          toolchain
          rust-analyzer-unwrapped
        ];

        shellHook = ''
          source scripts/auto.sh
        '';

        RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
      };

      formatter.${system} = pkgs.nixpkgs-fmt;
    };
}
