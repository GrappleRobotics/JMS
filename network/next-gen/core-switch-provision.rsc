/interface
  set ether1 comment="#jms-uplink"
  set ether2 comment="#jms-admin"
  set ether3 comment="#jms-server-primary"
  set ether4 comment="#jms-server-secondary"
  set ether5 comment="#jms-trunk-wifi"
  set ether6 comment="#jms-trunk-wifi"
  set ether7 comment="#jms-trunk-wifi"
  set ether8 comment="#jms-trunk-wifi"

  set ether9 comment="#jms-admin"
  set ether10 comment="#jms-imaging"
  set ether11 comment="#jms-admin"
  set ether12 comment="#jms-admin"
  set ether13 comment="#jms-admin"
  set ether14 comment="#jms-admin"
  set ether15 comment="#jms-admin"
  set ether16 comment="#jms-admin"

  set ether17 comment="#jms-trunk-blue"
  set ether18 comment="#jms-trunk-red"
  set ether19 comment="#jms-blue-1"
  set ether20 comment="#jms-red-1"
  set ether21 comment="#jms-blue-2"
  set ether22 comment="#jms-red-2"
  set ether23 comment="#jms-blue-3"
  set ether24 comment="#jms-red-3"

  set sfp-sfpplus1 comment="#jms-trunk-blue"
  set sfp-sfpplus2 comment="#jms-trunk-red"
  set sfp-sfpplus3 comment="#jms-admin"
  set sfp-sfpplus4 comment="#jms-admin"

/system/identity/set name="core-switch.admin.jms.local"

/interface/bridge/set name=bridge-trunk comment="#jms-trunk" [find]

:global teamvlans {
  "jms-blue-1"=10;
  "jms-blue-2"=20;
  "jms-blue-3"=30;
  "jms-red-1"=40;
  "jms-red-2"=50;
  "jms-red-3"=60;
  "jms-spare-1"=70;
  "jms-spare-2"=80;
  "jms-spare-3"=90;
}

/interface/bridge/vlan
  remove [find comment~"#jms-.*"]
  :global trunks ("bridge-trunk", [:toarray [/interface/ethernet/find where comment~"#jms-trunk" or comment~"#jms-server"]])

  add bridge=bridge-trunk vlan-ids=100 comment="#jms-admin" tagged="bridge-trunk"
  add bridge=bridge-trunk vlan-ids=99 comment="#jms-uplink" tagged="$trunks"
  add bridge=bridge-trunk vlan-ids=98 comment="#jms-imaging" tagged="$trunks"

  :foreach name,vlan in=$teamvlans do={
    add bridge=bridge-trunk vlan-ids=$vlan comment="#$name" untagged=[/interface/ethernet/find where comment~"#$name"] tagged="$trunks"
  }

/interface/bridge/port
  :foreach iface in=[/interface/ethernet/find where comment~"#jms-admin" or comment~"#jms-trunk" or comment~"#jms-server"] do={ set pvid=100 [find interface=[/interface/ethernet/get $iface name]] }
  :foreach iface in=[/interface/ethernet/find where comment~"#jms-uplink"] do={ set pvid=99 [find interface=[/interface/ethernet/get $iface name]] }
  :foreach iface in=[/interface/ethernet/find where comment~"#jms-imaging"] do={ set pvid=98 [find interface=[/interface/ethernet/get $iface name]] }

  :foreach name,vlan in=$teamvlans do={
    :foreach iface in=[/interface/ethernet/find where comment~"#$name"] do={ set pvid=$vlan [find interface=[/interface/ethernet/get $iface name]] }
  }

/interface/vlan
  remove [find comment~"#jms-.*"]
  add interface=[/interface/bridge/find] vlan-id=100 comment="#jms-admin" name="vlan-admin"
  add interface=[/interface/bridge/find] vlan-id=99 comment="#jms-uplink" name="vlan-uplink"
  # No need to assign a VLAN to jms-imaging, since it's not routed

  :foreach name,vlan in=$teamvlans do={
    add interface=[/interface/bridge/find] vlan-id=$vlan comment="#$name" name="vlan-$name"
  }

/ip/address
  remove [find comment~"#jms-.*"]
  add interface="vlan-admin" comment="jms-admin" address="10.0.100.1/24" network="10.0.100.0"

/interface/bridge/set vlan-filtering=yes pvid=100 bridge-trunk

/ip/dhcp-client
  remove [find comment~"#jms-.*"]
  add interface="vlan-uplink" comment="#jms-uplink"

/ip/pool
  remove [find comment~"#jms-.*"]
  add name="dhcp-admin" ranges=10.0.100.101-10.0.100.254 comment="#jms-admin"

/ip/dhcp-server
  remove [find comment~"#jms-.*"]
  add name="dhcp-admin" comment="#jms-admin" interface="vlan-admin" address-pool="dhcp-admin"

  network/remove [find comment~"#jms-.*"]
  network/add address=10.0.100.0/24 gateway=10.0.100.1 dns-server=10.0.100.1 comment="#jms-admin"

/ip/firewall
  nat/remove [find comment~"#jms-.*"]
  nat/add chain=srcnat out-interface="vlan-uplink" action="masquerade" comment="#jms-uplink"

/ip/dns set allow-remote-requests=yes

/ip/dns/static
  remove [find name~".*jms.local"]
  add address=10.0.100.5 name=jms.local type=A
  add address=10.0.100.10 name=jms-primary.jms.local type=A
  add address=10.0.100.11 name=jms-secondary.jms.local type=A
  add address=10.0.100.12 name=jms-tertiary.jms.local type=A