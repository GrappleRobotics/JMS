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