/interface
  set ether1 comment="#jms-trunk"
  set ether2 comment="#jms-admin"
  set ether3 comment="#jms-red-1"
  set ether4 comment="#jms-admin"
  set ether5 comment="#jms-red-2"
  set ether6 comment="#jms-admin"
  set ether7 comment="#jms-red-3"
  set ether8 comment="#jms-admin"

  set sfp9 comment="#jms-trunk"
  set sfp10 comment="#jms-red-1"
  set sfp11 comment="#jms-red-2"
  set sfp12 comment="#jms-red-3"

:global switchname "red-switch.admin.jms.local"
:global switchip 10.0.100.3
:global iscoreswitch false

/system/script/run common-provision
