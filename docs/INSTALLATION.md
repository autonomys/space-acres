# Installation

## General

Installation instructions common for all platforms.

### Hardware requirements

### CPU/RAM

Application is designed to run on relatively modern hardware. For best performance **Intel Skylake (6th gen Core)/AMD
Zen (Ryzen 1000) or newer processors** are strongly recommended, but Space Acres will run on older CPUs as well and will
automatically select suitable version of the software during start.

At least **4 physical cores and 8 GiB of RAM** is necessary for application to function, more CPU will make application
faster. If you have other software running at the same time, take that in consideration (it is not recommended to run on
a machine that has 8 GiB of physical RAM for everything).

While application will try to not use all of it all the time, it might crash if you don't have enough RAM and will not
farm properly if you don't have enough CPU.

### Storage

**HDDs are not supported and will never be**. Don't ask about it, don't try using smart caching, tiered storage or other
ways to accelerate it, you'll be 100% disappointed and just waste your time.

**Node will require 100 GiB of good quality SSD**. Doesn't have to be anything amazing, but something mid-range from a
reputable manufacturer with decent endurance is recommended.

**Farmer side can work with pretty much any SSD whatsoever that is not fake and not outright broken**, dedicating high
quality high endurance SSD is pointless unless you already have it for reasons unrelated to Subspace. Software writes to
disk in near-perfect for SSD way, effectively doing write leveling if SSD is solely dedicated to farming.

**RAID of any kind is pointless and can only harm** performance and/or rewards. RAID0 will most likely make things
slower rather than faster (application benefits from knowing underlying hardware topology). RAID1 or any other
redundancy level is 100% pointless too since farms are stateless and can be removed if disk breaks without losing data
on other disks, it'll just make thing slower and reduce effective capacity that can be used for farming, *reducing
farming rewards for literally no benefit in exchange*.

### Required ports

Application uses **TCP and UDP ports 30333 and 30433** for P2P communication with the rest of the network, both should
be open and exposed publicly on your router/firewall (settings for this are typically called "port forwarding"). Without
this application may sometimes work fine and sometimes have a hard time syncing or plotting, so it is
**strongly recommended**.

## Windows

For Windows go to [the latest release](https://github.com/subspace/space-acres/releases/latest) and download attached
file with `.msi` extension. It is not digitally signed, so you'll have to agree to accept the risk when downloading and
installing it for now. Note that while things might work on other versions of Windows, **only Windows 10 and 11** with
latest updates supported.

### Dependencies

In most cases you don't need to install anything else, but on fresh Windows installation Microsoft Visual C++
Redistributable might be missing, in which case application will show an error about some DLL files missing. If you see
that, install [Microsoft Visual C++ Redistributable packages for Visual Studio 2015, 2017, 2019, and 2022](https://learn.microsoft.com/en-US/cpp/windows/latest-supported-vc-redist?view=msvc-170#visual-studio-2015-2017-2019-and-2022)
for your architecture (most likely X64).

## Linux

Currently, there are two ways to get Space Acres on Linux:
* by installing `.deb` package on Ubuntu
* by running `.AppImage` bundle directly on any modern Linux distribution (including Ubuntu if you want to)

### Ubuntu

For **Ubuntu 22.04 or newer** (older versions not supported) go to [the latest release](https://github.com/subspace/space-acres/releases/latest) and download attached
file with `.deb` extension for your architecture (most likely `amd64`).

Then open terminal and run following commands to switch to downloads directory and install an app:
```bash
cd Downloads
sudo apt install ./space-acres*.deb
```

Replace `Downloads` with correct name of downloads directory if you have non-English Ubuntu installation or if you
downloaded file into a custom location. In case you have multiple versions of Space Acres downloaded, you might want to
replace `space-acres*.deb` with a full name of the file you've downloaded.

### Other Linux

For other distributions AppImage is available too, go to [the latest release](https://github.com/subspace/space-acres/releases/latest) and download attached
file with `.AppImage` extension for your architecture (most likely `x86_64`).

Then open terminal and run following commands to switch to downloads directory and make it executable:
```bash
cd Downloads
chmod +x space-acres-*.AppImage
```

After this either use `./space-acres-*.AppImage` in the terminal or double-click on the file in the file manager to open
the app. You may need to install FUSE library if you don't have it installed yet. In case you have multiple versions of
Space Acres downloaded, you might want to replace `space-acres-*.AppImage` with a full name of the file you've
downloaded.

There are no other Linux packages at the moment and if you build from source you hopefully know what
you are doing.
Consider contributing to Linux packaging though!

## macOS

macOS 14+ is supported on Apple Silicon hardware. Simply download `.dmg` file and install it as any other application.
