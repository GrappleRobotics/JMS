/interface
  set ether1 comment="#jms-trunk"
  set ether2 comment="#jms-admin"
  set ether3 comment="#jms-admin"
  set ether4 comment="#jms-admin"
  set ether5 comment="#jms-admin"
  set ether6 comment="#jms-admin"
  set ether7 comment="#jms-admin"
  set ether8 comment="#jms-admin"

  set sfp9 comment="#jms-trunk"
  set sfp10 comment="#jms-spare-1"
  set sfp11 comment="#jms-spare-2"
  set sfp12 comment="#jms-spare-3"

/system/identity/set name="spare-switch.admin.jms.local"

:global switchip "10.0.100.4/24"
:global teamvlans {
  "jms-spare-1"=70;
  "jms-spare-2"=80;
  "jms-spare-3"=90
}

/interface/bridge/set name=bridge-trunk comment="#jms-trunk" [find]

/interface/bridge/vlan
  remove [find comment~"#jms-.*"]
  :global trunks ("bridge-trunk", [:toarray [/interface/ethernet/find where comment~"#jms-trunk"]])

  add bridge=bridge-trunk vlan-ids=100 comment="#jms-admin" tagged="bridge-trunk"

  :foreach name,vlan in=$teamvlans do={
    add bridge=bridge-trunk vlan-ids=$vlan comment="#$name" tagged="$trunks"
  }

/interface/bridge/port
  :foreach iface in=[/interface/ethernet/find where comment~"#jms-admin" or comment~"#jms-trunk"] do={ set pvid=100 [find interface=[/interface/ethernet/get $iface name]] }

  :foreach name,vlan in=$teamvlans do={
    :foreach iface in=[/interface/ethernet/find where comment~"#$name"] do={ set pvid=$vlan [find interface=[/interface/ethernet/get $iface name]] }
  }

/interface/vlan
  remove [find comment~"#jms-.*"]
  add interface=[/interface/bridge/find] vlan-id=100 comment="#jms-admin" name="vlan-admin"

  :foreach name,vlan in=$teamvlans do={
    add interface=[/interface/bridge/find] vlan-id=$vlan comment="#$name" name="vlan-$name"
  }

/ip/address
  remove [find comment~"#jms-.*"]
  add interface="vlan-admin" comment="jms-admin" address=$switchip network="10.0.100.0"

/ip/route
  remove [find comment~"#jms-.*"]
  add gateway="10.0.100.1" comment="#jms-gateway"

/interface/bridge/set vlan-filtering=yes pvid=100 bridge-trunk
