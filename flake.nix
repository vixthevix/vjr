{
  description = "ESP32 Rust Development Environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        
        # Needed for raylib
        guiLibs = with pkgs; [
          libGL
          alsa-lib
          xorg.libX11
          xorg.libXrandr
          xorg.libXinerama
          xorg.libXcursor
          xorg.libXi
          wayland
          libxkbcommon
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust compiler and cargo (main tool)
            rustup
            
            # ESP32 specific tools
            espup          # Downloads custom ESP Rust compiler
            espflash       # Flashes Rust code into ESP32
            probe-rs-tools # Debugging tools (Unused)
            
            # System dependencies
            pkg-config # Provides help for locating C libraries used in Rust crates
            openssl    # Provides ease-of-use library for secure HTTPS connections.
            systemd    # Provides libudev needed by probe-rs
            libusb1    # Provides functionality for probe-rs to communicate with ESP32
            
            # Helper tool dependencies
            gcc     # Provides C compiler
            gnumake # Provides build system
            raylib  # Provides PC graphical library
          ] ++ guiLibs;

          shellHook = ''
            
            fun_display() {
                local str="$1"
                local padding="$2"
                if [[ -z "$str" || -z "$padding" ]]; then
                    return 1
                fi

                local str_len=''${#str}
                local row_count=''$((str_len + padding))
                local i

                echo
                
                echo -n " "
                for ((i = 0 ; i < $row_count ; i++)) do
                    echo -n "-"
                done
                echo " "

                # Assume 1 line string because yeah
                echo "| $str |"
                
                echo -n " "
                for ((i = 0 ; i < $row_count ; i++)) do
                    echo -n "-"
                done
                echo " "
                
                echo
                
                return 0
            }

            # Ensure local cargo binaries are accessible (if they exist)
            export PATH="$HOME/.cargo/bin:$PATH"
            
            # Location of espup environmental script
            env_script="$HOME/export-esp.sh"
            
            # Has rustup been run? (needed for cargo)
            if [ ! rustup default &> /dev/null ]; then
                fun_display "Rust and cargo not configured. Run 'rustup default stable' to proceed normally." 2
            # Has espup been run?
            elif [ ! -f $env_script ]; then
                fun_display "$env_script not found, run 'espup install' and '. \$HOME/export-esp.sh' to proceed normally." 5
            else
                # Auto boot into ESP development environment
                . $env_script
                fun_display "ESP32 DEVELOPMENT ENVIRONMENT LOADED" 3 
            fi
          '';
        };
      }
    );
}
