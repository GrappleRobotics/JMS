Network
=======

This directory contains a bunch of configurations for your networking equipment. 

# Field Access Point
If you're using a segmented network (like the full FMS, with VLANs), you'll want to the use the field access point configurations.

**IMPORTANT:** These configurations are based on the FIRST-provided OpenWRT distribution (get it [here](https://usfirst.collab.net/sf/frs/do/viewRelease/projects.offseasonfms/frs.2017_fms_offseason.2017_linksys_ap_image_with_vlan)). 
- _If you're using a stock WRT1900ACS AP, you can upload this image from the WebUI at http://192.168.1.1_

It may also be possible to use a stock OpenWRT image and apply these configurations from scratch, but I have not yet tried this method.

### `field-ap-wrt1900acs.tar.gz`
This is the same as the OpenWRT Offseason VLAN image from FIRST (above), but with some key differences:
- Country code switched to "AU" for use in Australia
- Make router admin accessible over the WAN interface (tagged on VLAN1, `192.168.1.1` - bound to LAN bridge in WRT)
- VLAN100 must be tagged on WAN interface (trunk line)

Upload at `http://192.168.1.1` on an OpenWRT installation running LuCI, or extract to `/etc` via SSH and reboot.
