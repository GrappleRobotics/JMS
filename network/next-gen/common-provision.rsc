:global switchname
:global switchip
:global iscoreswitch

:global vlanmap {
  "jms-blue-1"=10;
  "jms-blue-2"=20;
  "jms-blue-3"=30;
  "jms-red-1"=40;
  "jms-red-2"=50;
  "jms-red-3"=60;
  "jms-spare-1"=70;
  "jms-spare-2"=80;
  "jms-spare-3"=90;
  "jms-uplink"=99;
  "jms-admin"=100;
}

/system/identity/set name=$switchname
/interface/bridge/set name=bridge-trunk comment="#jms-trunk" [find]

/interface/bridge/vlan
  remove [find comment~"#jms-.*"]
  :global trunks ("bridge-trunk", [:toarray [/interface/ethernet/find where comment~"#jms-trunk"]])

  :foreach iname,vlan in=$vlanmap do={
    add bridge=bridge-trunk vlan-ids=$vlan comment="#$iname" tagged="$trunks"
  }

/interface/bridge/port
  :foreach iface in=[/interface/ethernet/find where comment~"#jms-.*"] do={ remove [find interface=[/interface/ethernet/get $iface name]] }
  :foreach iface in=[/interface/ethernet/find where comment~"#jms-trunk"] do={ add bridge=bridge-trunk pvid=100 interface=[/interface/ethernet/get $iface name] }

  :foreach iname,vlan in=$vlanmap do={
    :foreach iface in=[/interface/ethernet/find where comment="#$iname"] do={ add bridge=bridge-trunk pvid=$vlan interface=[/interface/ethernet/get $iface name] frame-types=admit-only-untagged-and-priority-tagged }
  }

/interface/vlan
  remove [find comment~"#jms-.*"]

  :foreach iname,vlan in=$vlanmap do={
    add interface=[/interface/bridge/find] vlan-id=$vlan comment="#$iname" name="vlan-$iname"
  }

/ip/address
  remove [find comment~"#jms-.*"]
  add interface=[/interface/vlan/find where comment~"#jms-admin"] comment="jms-admin" address="$switchip/24" network="10.0.100.0"

/interface/bridge/set vlan-filtering=yes pvid=100 bridge-trunk

:if ($iscoreswitch = false) do={
  /ip/route
    remove [find comment~"#jms-.*"]
    add gateway="10.0.100.1" comment="#jms-gateway"
} else={
  /ip/dhcp-client
    remove [find comment~"#jms-.*"]
    add interface=[/interface/vlan/find where comment~"#jms-uplink"] comment="#jms-uplink"
  
  /ip/pool
    remove [find comment~"#jms-.*"]
    add name="dhcp-admin" ranges=10.0.100.101-10.0.100.254 comment="#jms-admin"

  /ip/dhcp-server
    remove [find comment~"#jms-.*"]
    add name="dhcp-admin" comment="#jms-admin" interface=[/interface/vlan/find where comment~"#jms-admin"] address-pool="dhcp-admin"

    network/remove [find comment~"#jms-.*"]
    network/add address=10.0.100.0/24 gateway=$switchip dns-server=$switchip comment="#jms-admin"
  
  /ip/firewall
    nat/remove [find comment~"#jms-.*"]
    nat/add chain=srcnat out-interface=[/interface/vlan/find where comment~"#jms-uplink"] action="masquerade" comment="#jms-uplink"
  
  /ip/dns set allow-remote-requests=yes

  /ip/dns/static
    remove [find name~".*jms.local"]
    add address=10.0.100.5 name=jms.local type=A
    add address=10.0.100.10 name=jms-primary.jms.local type=A
    add address=10.0.100.11 name=jms-secondary.jms.local type=A
    add address=10.0.100.12 name=jms-tertiary.jms.local type=A
}