A local docker container for postgres must be run: 
```bash
docker run --name postgres -e POSTGRES_PASSWORD=postgres -p 127.0.0.1:5432:5432 -d postgres
```

## DHCP & Radios
Looks like the FRC Radios start their own DHCP server, I assume in the range of .200+ - when bridged, only influences wired interfaces.

That DHCP server serves the robot itself, while the FMS also runs a DHCP server for Driver Stations (let's go with .100 - .150 for each team)

Robot radio has address .1
Field router has address .4 (except on admin, where we use .1 for convention)

The RIO has a MAC prefix of 00:80:2F (for all NI devices).

## Permissions
### Polkit DBUS
`/etc/polkit-1/localauthority/50-local.d/10-jms.pkla`
```
[systemd]
Identity=unix-group:jaci
Action=org.freedesktop.systemd1.manage-units
ResultAny=yes
ResultInactive=yes
ResultActive=yes
```

Required to give `jaci` user access to system dbus to stop/start/reload services without sudo. Should change into a group for final deployment. Ansible might be suitable to make this work.

## Radio Configuration (manual)
My laptop doesn't support the FRC radio programmer (external USB-C Ethernet adapter isn't compatible), so I configure FRC OpenMesh
radios with the below for testing. Confirm it on the actual programmer before the event.

Based on the FRC WPA Kiosk:

```
mode,team,ssid,key,firewall,bwlimit,dhcp,chan2.4,chan5,comment,date
```
- `mode`: One of: `["B5", "AP24", "AP5", "B24"]` - on the field we want B5
- `team`: Team number
- `ssid`: SSID. On the field, it's the team number
- `key`: WPA Key
- `firewall/bwlimit/dhcp`: Y or N. For the field, this should be N,Y,Y (firewall is managed by field radio, but it shouldn't matter)
- `chan2.4,chan5`: WiFi channels. 0,0 looks to be the default, which I assume is auto
- `comment,date`: This is probably the event code and end datetime, since the radio programmer will prevent teams from flashing radios while the event is ongoing. As far as I can tell, these have no bearing on radio operation

The programmer listens on `192.168.1.1:8888`. After it sends its front matter, send the above followed by a newline. Default computer IP is `192.168.1.51`.

Can also flash with `./ap51-flash <iface> <image>`. Image files can be found in the install dir of the FRC WPA Kiosk.