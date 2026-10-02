{
  description = "A flake for a very basic physics engine";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";
  };

  outputs = inputs: let
    system = "x86_64-linux";
    pkgs = import inputs.nixpkgs { inherit system; };
  in {
        devShells.${system}.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
                cargo
                rust-analyzer
                rustfmt
                rustc
                gdb
            ];

            buildInputs = with pkgs; [
                wayland
                libxkbcommon
                vulkan-loader
                vulkan-tools
                libGL
            ];
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
                pkgs.wayland
                pkgs.libxkbcommon
                pkgs.vulkan-loader
                pkgs.libGL
            ];
        };
  };
}
