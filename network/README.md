Network
=======

This directory contains a bunch of configurations for your networking equipment. 

# Field Access Point
If you're using a segmented network (like the full FMS, with VLANs), you'll want to the use the field access point configurations.

### `field-ap-wrt1900acs-19.07.8.tar.gz`
This is the custom configuration for OpenWRT 19.07.8 (get it [here](https://firmware-selector.openwrt.org/?version=19.07.8&target=mvebu%2Fcortexa9&id=linksys_wrt1900acs)).

You can flash this image on `192.168.1.1`. Similarly, you can load the .tar.gz configuration from this same URL after OpenWRT is loaded through LuCI (or through SSH). 

This configuration includes many of the same changes as below.

### `field-ap-wrt1900acs.tar.gz`
**IMPORTANT:** These configurations are based on the FIRST-provided OpenWRT distribution (get it [here](https://usfirst.collab.net/sf/frs/do/viewRelease/projects.offseasonfms/frs.2017_fms_offseason.2017_linksys_ap_image_with_vlan)). 

Note also that we have observed some instability issues with this image. It is recommended to use the new image above.

- _If you're using a stock WRT1900ACS AP, you can upload this image from the WebUI at http://192.168.1.1_
This is the same as the OpenWRT Offseason VLAN image from FIRST (above), but with some key differences:
- Country code switched to "AU" for use in Australia
- Make router admin accessible over the WAN interface (tagged on VLAN1, `192.168.1.1` - bound to LAN bridge in WRT)
- VLAN100 must be tagged on WAN interface (trunk line)

Upload at `http://192.168.1.1` on an OpenWRT installation running LuCI, or extract to `/etc` via SSH and reboot.
