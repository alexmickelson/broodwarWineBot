{
  description = "Broodwar Wine Bot - StarCraft Broodwar AI bot with Rust and BWAPI";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "x86_64-pc-windows-gnu" ];
          extensions = [ "rust-src" ];
        };

        mingwPkgs = pkgs.pkgsCross.mingwW64;
        mingwCC = mingwPkgs.stdenv.cc;
        
        # Get GCC version dynamically
        gccVersion = mingwPkgs.stdenv.cc.cc.version;
        
        buildInputs = with pkgs; [
          rustToolchain
          mingwCC
          mingwPkgs.windows.pthreads
        ];

        nativeBuildInputs = with pkgs; [
          pkg-config
          clang
          llvmPackages.libclang
        ];

        shellEnv = {
          CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${mingwCC}/bin/x86_64-w64-mingw32-gcc";
          CC_x86_64_pc_windows_gnu = "${mingwCC}/bin/x86_64-w64-mingw32-gcc";
          CXX_x86_64_pc_windows_gnu = "${mingwCC}/bin/x86_64-w64-mingw32-g++";
          AR_x86_64_pc_windows_gnu = "${mingwCC}/bin/x86_64-w64-mingw32-ar";
          
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          
          # Bindgen configuration for MinGW cross-compilation
          # Tell bindgen to use mingw headers, not Linux headers
          BINDGEN_EXTRA_CLANG_ARGS = pkgs.lib.concatStringsSep " " [
            "--target=x86_64-w64-mingw32"
            # Use -isystem to add includes with lower priority than -I
            # This allows the mingw headers to find clang intrinsics
            "-isystem${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.llvmPackages.libclang.version}/include"
            "-isystem${mingwCC.cc}/include/c++/${gccVersion}"
            "-isystem${mingwCC.cc}/include/c++/${gccVersion}/x86_64-w64-mingw32"
            "-isystem${mingwCC.cc}/include/c++/${gccVersion}/backward"
            "-isystem${mingwPkgs.windows.mingw_w64_headers}/include"
            "-isystem${mingwPkgs.windows.pthreads}/include"
            "-D_WIN32"
            "-D_WIN64"
          ];
          
          # Set target for bindgen
          TARGET = "x86_64-pc-windows-gnu";
        };

        # Build script
        buildScript = pkgs.writeShellScriptBin "build-rustbot" ''
          set -e
          cd rustbot
          
          echo "Building Rust bot for Windows (x86_64-pc-windows-gnu)..."
          cargo build --target x86_64-pc-windows-gnu --release
          
          if [ -f "target/x86_64-pc-windows-gnu/release/rustbot.exe" ]; then
            echo "✓ Build successful: rustbot/target/x86_64-pc-windows-gnu/release/rustbot.exe"
          else
            echo "✗ Build failed!"
            exit 1
          fi
        '';

        buildDebugScript = pkgs.writeShellScriptBin "build-rustbot-debug" ''
          set -e
          cd rustbot
          
          echo "Building Rust bot for Windows (debug)..."
          cargo build --target x86_64-pc-windows-gnu
          
          if [ -f "target/x86_64-pc-windows-gnu/debug/rustbot.exe" ]; then
            echo "✓ Build successful: rustbot/target/x86_64-pc-windows-gnu/debug/rustbot.exe"
          else
            echo "✗ Build failed!"
            exit 1
          fi
        '';

        cleanScript = pkgs.writeShellScriptBin "clean-rustbot" ''
          cd rustbot
          cargo clean
          echo "✓ Cleaned build artifacts"
        '';

        checkScript = pkgs.writeShellScriptBin "check-rustbot" ''
          cd rustbot
          cargo check --target x86_64-pc-windows-gnu
        '';

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = buildInputs ++ nativeBuildInputs ++ [
            buildScript
            buildDebugScript
            cleanScript
            checkScript
            
            # Additional development tools
            pkgs.cargo-watch
            pkgs.cargo-edit
            pkgs.rust-analyzer
            pkgs.wine64
          ];

          shellHook = ''
            echo "🤖 Broodwar Wine Bot Development Environment"
            echo "==========================================="
            echo ""
            echo "Available commands:"
            echo "  build-rustbot        - Build release version for Windows"
            echo "  build-rustbot-debug  - Build debug version for Windows"
            echo "  check-rustbot        - Quick check without building"
            echo "  clean-rustbot        - Clean build artifacts"
            echo ""
            echo "Manual build:"
            echo "  cd rustbot && cargo build --target x86_64-pc-windows-gnu"
            echo ""
            echo "Environment configured for cross-compilation to Windows"
            echo "Target: x86_64-pc-windows-gnu"
            echo ""
            
            ${pkgs.lib.concatStringsSep "\n" 
              (pkgs.lib.mapAttrsToList 
                (name: value: "export ${name}=\"${value}\"") 
                shellEnv)}
          '';
        };

        packages = {
          # Build the Windows executable
          rustbot = pkgs.stdenv.mkDerivation {
            pname = "rustbot";
            version = "0.1.0";
            src = ./rustbot;

            nativeBuildInputs = nativeBuildInputs ++ buildInputs;

            buildPhase = ''
              export HOME=$TMPDIR
              ${pkgs.lib.concatStringsSep "\n" 
                (pkgs.lib.mapAttrsToList 
                  (name: value: "export ${name}=\"${value}\"") 
                  shellEnv)}
              
              cargo build --release --target x86_64-pc-windows-gnu --locked
            '';

            installPhase = ''
              mkdir -p $out/bin
              cp target/x86_64-pc-windows-gnu/release/rustbot.exe $out/bin/
            '';
          };

          default = self.packages.${system}.rustbot;
        };

        apps = {
          build = {
            type = "app";
            program = "${buildScript}/bin/build-rustbot";
          };
          
          build-debug = {
            type = "app";
            program = "${buildDebugScript}/bin/build-rustbot-debug";
          };

          clean = {
            type = "app";
            program = "${cleanScript}/bin/clean-rustbot";
          };

          check = {
            type = "app";
            program = "${checkScript}/bin/check-rustbot";
          };

          default = self.apps.${system}.build;
        };
      }
    );
}
