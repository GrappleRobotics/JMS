/interface
  set ether1 comment="#jms-uplink"
  set ether2 comment="#jms-admin"
  set ether3 comment="#jms-blue-1"
  set ether4 comment="#jms-blue-2"
  set ether5 comment="#jms-blue-3"
  set ether6 comment="#jms-red-1"
  set ether7 comment="#jms-red-2"
  set ether8 comment="#jms-red-3"

:global switchname "core-switch.admin.jms.local"
:global switchip 10.0.100.1
:global iscoreswitch true

/system/script/run common-provision