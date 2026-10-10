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
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust compiler and cargo (main tool)
            rustup
            
            # ESP32 specific tools
            espup    # Downloads custom ESP Rust compiler
            espflash # Flashes Rust code into ESP32
            probe-rs # Debugging tools
            
            # System dependencies
            pkg-config # Provides help for locating C libraries used in Rust crates
            openssl    # Provides ease-of-use library for secure HTTPS connections.
            systemd    # Provides libudev needed by probe-rs
            libusb1    # Provides functionality for probe-rs to communicate with ESP32
          ];

          shellHook = ''
            
            fun_display() {
                local str=$1
                if [ -z $str ]; then
                    return 1
                fi

                local str_len=''${#str}
                local i

                echo
                
                echo -n " "
                for ((i = 0 ; i < $str_len + 2 ; i++)) do
                    echo -n "-"
                done
                echo " "

                # Assume 1 line string because yeah
                echo "| $str |"
                
                echo -n " "
                for ((i = 0 ; i < $str_len + 2 ; i++)) do
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
            
            # Auto boot into ESP development environment
            if [ -f $env_script ]; then
                . $env_script
                fun_display "ESP32 DEVELOPMENT ENVIRONMENT LOADED" 
            
            
            else
                fun_display "$env_script not found, run 'espup install' and '. \$HOME/export-esp.sh' to proceed normally." 
            fi
          '';
        };
      }
    );
}
