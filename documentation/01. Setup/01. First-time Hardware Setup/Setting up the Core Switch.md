This guide assumes you are using the TP-Link SG1016DE as the Core JMS switch (the same switch found in Grapple's JMS-in-a-box).

# 1. Connect to the Switch
Connect to the switch via Ethernet

# 2. Set your IP Address
On your computer, set your IP Address to 192.168.0.50
![[switch.2.1.png]]

# 3. Open the Switch Web UI
Navigate to 192.168.0.1 in a web browser. Enter the default credentials of admin / admin. You will be prompted to change your password. Choose a secure one and write it down, or use the default of "jmsR0cks".

# 4. Load the Configuration
Go to System > System Tools > Backup and Restore and load the configuration (JMS/network/core-switch-sg1016de.cfg)

![[switch.4.1.png]]

*Note: Unfortunately, the SG1016DE doesn't support changing the administration VLAN, so Port 1 is reserved for configuring the network switch*

# 5. Verify Ports
Go to Switching > Port Setting and verify that each port works by plugging a new device into each port and cross-referencing with the SG1016DE configuration diagram (JMS/network/core-switch-configuration-sg1016de.png)

