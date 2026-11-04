# vjr
A handheld game console, based on the ESP32 and Rust programming language.

## Hardware

### Requirements

The vjr follows these hardware specs:

| Part | Details |
| ----- | -------- |
| Microcontroller | [ESP32 WROOM 32 ](https://documentation.espressif.com/esp32-wroom-32_datasheet_en.html) |
| Screen | [3.2" 320x240px ILI9341 SPI Screen](https://www.aliexpress.com/item/1005006390568265.html?spm=a2g0o.order_list.order_list_main.22.69de1802Nt2bCpl) |
| Digital buttons | [Tactile buttons](https://thepihut.com/products/180-piece-ultimate-tactile-button-kit) |
| Analogue sticks | [Nintendo 3DS Analogue Joysticks](https://www.aliexpress.com/item/1005009784755399.html?spm=a2g0o.order_list.order_list_main.16.69de1802Nt2bCp) |
| Speakers | TBD |
| SD card reader | [Adafruit Micro SD SDIO Card Reader](https://www.adafruit.com/product/4682) |

### Schematics

To be updated as the project goes on.

## Software

### Requirements

**Traditional**

Before you can build the vjr firmware, run:

- **Debian/Ubuntu/Mint**
  ```
  sudo apt install build-essential git make wget tar openssl libgl1-mesa-dev libx11-dev zenity iproute2 iw rfkill iptables hostapd dnsmasq
  ```
- **Arch**
  ```
  sudo pacman -S base-devel git make wget tar openssl zenity iproute2 iw rfkill iptables hostapd dnsmasq
  ```
- **Fedora**
  ```
  sudo dnf install @development-tools git wget tar openssl mesa-libGL-devel libX11-devel iproute iw rfkill iptables hostapd dnsmasq
  ```

**Using [Nix](https://nixos.org/)**

The project holds a [Nix flake file](https://github.com/vixthevix/vjr/blob/main/flake.nix) for creating a development environment with Nix.

To use it, follow the Nix installation guide (if not yet installed), then run:

``` shell
nix develop
```

### Build

To compile and flash the firmware onto the ESP32, run:

``` shell
cargo run --release
```
If using the Nix flake, ensure you are in the development environment beforehand.


## How to use the system

To be updated as the project goes on.

## More information

To be updated as the project goes on.
