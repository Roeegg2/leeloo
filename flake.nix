{
  description = "Development environment for the Leeloo x86-64 kernel";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          # OVMF is x86-64 firmware; pull from x86_64-linux if developing on aarch64
          ovmf = if system == "x86_64-linux"
                 then pkgs.OVMF
                 else nixpkgs.legacyPackages.x86_64-linux.OVMF;
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              zig
              zls
              limine-full
              qemu
              xorriso
              mtools
              dosfstools
              gdb
            ];

            LIMINE_DIR = "${pkgs.limine-full}/share/limine";
            OVMF_FD = "${ovmf.fd}/FV/OVMF.fd";
          };
        });
    };
}
