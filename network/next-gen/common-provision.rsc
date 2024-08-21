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
  "jms-admin"=1;
  "jms-guest"=200;
}

:global dhcpmap {
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

/system/identity/set name=$switchname
/interface/bridge/set name=bridge-trunk comment="#jms-trunk" [find]

/interface/bridge/vlan
  remove [find comment~"#jms-.*"]
  :global trunks ("bridge-trunk", [:toarray [/interface/ethernet/find where comment~"#jms-trunk"]])

  :foreach iname,vlan in=$vlanmap do={
    :if ($iname = "jms-admin") do={
      add bridge=bridge-trunk vlan-ids=$vlan comment="#$iname" tagged="bridge-trunk"
    } else={
      add bridge=bridge-trunk vlan-ids=$vlan comment="#$iname" tagged="$trunks"
    }
  }

/interface/bridge/port
  :foreach iface in=[/interface/ethernet/find where comment~"#jms-.*"] do={ remove [find interface=[/interface/ethernet/get $iface name]] }
  :foreach iface in=[/interface/ethernet/find where comment~"#jms-trunk"] do={ add bridge=bridge-trunk pvid=1 interface=[/interface/ethernet/get $iface name] }

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
  add interface=[/interface/vlan/find where comment~"#jms-admin"] comment="#jms-admin" address="$switchip/24" network="10.0.100.0"

/interface/bridge/set vlan-filtering=yes pvid=1 bridge-trunk

:if ($iscoreswitch = false) do={
  /ip/route
    remove [find comment~"#jms-.*"]
    add gateway="10.0.100.1" comment="#jms-gateway"
} else={
  /ip/dhcp-client
    remove [find comment~"#jms-.*"]
    add interface=[/interface/vlan/find where comment~"#jms-uplink"] comment="#jms-uplink"
  
  /ip/address
    add interface=[/interface/vlan/find where comment~"#jms-guest"] comment="#jms-guest" address="10.0.200.1/24" network="10.0.200.0"
    :foreach iname,vlan in=$dhcpmap do={
      add interface=[/interface/vlan/find where comment~"#$iname"] comment="#$iname" address="10.0.1$vlan.4/24" network="10.0.1$vlan.0"
    }

  /ip/pool
    remove [find comment~"#jms-.*"]
    add name="dhcp-jms-admin" ranges=10.0.100.101-10.0.100.254 comment="#jms-admin"
    add name="dhcp-jms-guest" ranges=10.0.200.101-10.0.200.254 comment="#jms-guest"

    :foreach iname,vlan in=$dhcpmap do={
      add name="dhcp-$iname" ranges="10.0.1$vlan.100-10.0.1$vlan.150" comment="#$iname"
    }

  /ip/dhcp-server
    remove [find comment~"#jms-.*"]
    add name="dhcp-admin" comment="#jms-admin" interface=[/interface/vlan/find where comment~"#jms-admin"] address-pool="dhcp-jms-admin"
    add name="dhcp-guest" comment="#jms-guest" interface=[/interface/vlan/find where comment~"#jms-guest"] address-pool="dhcp-jms-guest"

    :foreach iname,vlan in=$dhcpmap do={
      add name="dhcp-$iname" comment="#$iname" interface=[/interface/vlan/find where comment~"#$iname"] address-pool="dhcp-$iname"
    }

    network/remove [find comment~"#jms-.*"]
    network/add address=10.0.100.0/24 gateway=$switchip dns-server=$switchip comment="#jms-admin"
    network/add address=10.0.200.0/24 gateway="10.0.200.1" dns-server="10.0.200.1" comment="#jms-guest"

    :foreach iname,vlan in=$dhcpmap do={
      network/add comment="#$iname" address="10.0.1$vlan.0/24"
    }
  
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
  
  /ip/firewall/filter
    remove [find comment~"#jms.*"]

    add comment="#jms-existing" chain=input action=accept connection-state=established,related
    add comment="#jms-existing" chain=forward action=accept connection-state=established,related
    add comment="#jms-admin-router-access" chain=input action=accept src-address=10.0.100.0/24
    add comment="#jms-guest-router-access" chain=input action=accept src-address=10.0.200.0/24
    add comment="#jms-admin-internet-access" chain=forward action=accept src-address=10.0.100.0/24 out-interface=[/interface/vlan/find where comment~"#jms-uplink"]
    add comment="#jms-guest-internet-access" chain=forward action=accept src-address=10.0.200.0/24 out-interface=[/interface/vlan/find where comment~"#jms-uplink"]
    add comment="#jms-icmp-router" chain=input action=accept protocol=icmp
    add comment="#jms-deny-fms-guest" chain=forward action=drop dst-address=10.0.100.5 src-address=10.0.200.0/24
    add comment="#jms-ds2fms-tcp" chain=forward action=accept protocol=tcp dst-address=10.0.100.5 dst-port=1750
    add comment="#jms-ds2fms-udp" chain=forward action=accept protocol=udp dst-address=10.0.100.5 dst-port=1160
    add comment="#jms-ds2fms-icmp" chain=forward action=accept protocol=icmp dst-address=10.0.100.5
    add comment="#jms-fms2ds-udp" chain=forward action=accept protocol=udp src-address=10.0.100.5 dst-port=1121
    add comment="#jms-default-deny" chain=input action=drop
    add comment="#jms-default-deny" chain=forward action=drop
}