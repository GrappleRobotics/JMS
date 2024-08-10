/interface
  set ether1 comment="#jms-trunk"
  set ether2 comment="#jms-admin"
  set ether3 comment="#jms-blue-1"
  set ether4 comment="#jms-admin"
  set ether5 comment="#jms-blue-2"
  set ether6 comment="#jms-admin"
  set ether7 comment="#jms-blue-3"
  set ether8 comment="#jms-admin"

  set sfp9 comment="#jms-trunk"
  set sfp10 comment="#jms-blue-1"
  set sfp11 comment="#jms-blue-2"
  set sfp12 comment="#jms-blue-3"

:global switchname "blue-switch.admin.jms.local"
:global switchip 10.0.100.2
:global iscoreswitch false

/system/script/run common-provision
