/interface
  set ether1 comment="#jms-uplink"
  set ether2 comment="#jms-admin"
  set ether3 comment="#jms-admin"
  set ether4 comment="#jms-admin"
  set ether5 comment="#jms-trunk"
  set ether6 comment="#jms-trunk"
  set ether7 comment="#jms-trunk"
  set ether8 comment="#jms-trunk"

  set ether9 comment="#jms-admin"
  set ether10 comment="#jms-imaging"
  set ether11 comment="#jms-admin"
  set ether12 comment="#jms-admin"
  set ether13 comment="#jms-admin"
  set ether14 comment="#jms-admin"
  set ether15 comment="#jms-admin"
  set ether16 comment="#jms-admin"

  set ether17 comment="#jms-trunk"
  set ether18 comment="#jms-trunk"
  set ether19 comment="#jms-blue-1"
  set ether20 comment="#jms-red-1"
  set ether21 comment="#jms-blue-2"
  set ether22 comment="#jms-red-2"
  set ether23 comment="#jms-blue-3"
  set ether24 comment="#jms-red-3"

  set sfp-sfpplus1 comment="#jms-trunk"
  set sfp-sfpplus2 comment="#jms-trunk"
  set sfp-sfpplus3 comment="#jms-admin"
  set sfp-sfpplus4 comment="#jms-admin"

:global switchname "core-switch.admin.jms.local"
:global switchip 10.0.100.1
:global iscoreswitch true

/system/script/run common-provision