# 0. Requirements
For JMS, we require the same model of radio as the official field, but with a different firmware setup. This radio is the Linksys WRT1900ACS V2, and can be bought online. We will be flashing OpenWRT on it and loading a custom configuration to make it compatible with JMS.

# 1. Download OpenWRT

Download OpenWRT 22.03.0 for the Linksys WRT1900ACS V2: https://firmware-selector.openwrt.org/?version=22.03.0&target=mvebu%2Fcortexa9&id=linksys_wrt1900acs

# 2. Flash OpenWRT to the Radio
Unfortunately we don't have screenshots for this step, but the next step is to perform a firmware update on the WRT1900ACSV2, supplying the OpenWRT image above.

# 3. Connect to the Radio
Connect via ethernet to any of the "LAN" ports on the rear of the radio

# 4. Load the configuration
Open a browser to 192.168.1.1 and go System > Backup / Flash Firmware and restore the archive provided with JMS (`JMS/network/field-ap-wrt1900acsv2-22.03.0.tar.gz`)

![[radio.4.1.png]]

Once the archive is loaded, you should be good to go. Plug the WAN port of the radio into the RADIO port on your core switch. 